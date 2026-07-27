//! Database storage for app registrations.

use std::{
   collections::HashSet,
   path::Path,
};

use anyhow::{
   Context as _,
   Result,
};
use rusqlite::params;
use tokio_rusqlite::Connection;

use crate::{
   auth::verify_secret,
   types::{
      AppId,
      ConnectorToken,
      Cursor,
      InstallId,
      InstallSecret,
      MessageId,
      MessageKind,
   },
};

const SCHEMA_VERSION: i32 = 6;

/// Enrollment is open by design, so these ceilings are the only thing bounding
/// a minted `install_id`. All are far above real usage.
pub const MAX_REGISTRATIONS_PER_INSTALL: usize = 64;
pub const MAX_UNIFIED_PUSH_PER_INSTALL: usize = 64;
pub const MAX_OUTBOX_ROWS_PER_INSTALL: usize = 8192;
/// Each registration is one MCS socket to Google, so this is the real capacity
/// bound on the process, not just a row count.
pub const MAX_TOTAL_REGISTRATIONS: usize = 4096;
pub const MAX_TOTAL_INSTALLATIONS: usize = 1024;

/// An install that has never registered anything is an abandoned or hostile
/// claim. It re-authenticates to refresh `updated_at`, so only `created_at`
/// retires it.
pub const UNUSED_INSTALL_MAX_AGE_DAYS: u32 = 7;

/// Kept distinct from an ownership failure, which is the client's fault.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QuotaRejection {
   Install,
   Server,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UnifiedPushClaim {
   Registered(String),
   /// The connector token is already bound to a different package on this
   /// install, which a well-behaved distributor never asks for.
   TokenOwnedByAnotherApp,
   Refused(QuotaRejection),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Claim {
   /// The `install_id` did not exist and was enrolled by this call.
   Enrolled,
   /// The `install_id` existed and the secret matched.
   Existing,
   /// The `install_id` existed under a different secret.
   Denied,
   Refused(QuotaRejection),
}

#[derive(Debug, Clone)]
pub struct Registration {
   pub install_id:          InstallId,
   pub app_id:              AppId,
   pub secret_hash:         String,
   pub fcm_token:           Option<String>,
   pub firebase_app_id:     String,
   pub firebase_project_id: String,
   pub firebase_api_key:    String,
   pub cert_sha1:           Option<String>,
   pub app_version:         Option<i32>,
   pub app_version_name:    Option<String>,
   pub target_sdk:          Option<i32>,
}

#[derive(Debug, Clone)]
pub struct OutboxMessage {
   pub id:              MessageId,
   pub app_id:          AppId,
   pub kind:            MessageKind,
   pub connector_token: Option<ConnectorToken>,
   pub payload:         Vec<u8>,
   pub attempts:        u32,
}

#[derive(Debug, Clone)]
pub struct UnifiedPushRegistration {
   pub install_id:      InstallId,
   pub app_id:          AppId,
   pub connector_token: ConnectorToken,
   pub vapid_pubkey:    Option<String>,
}

#[derive(Debug)]
pub struct PruneOutcome {
   pub pairs:    Vec<(InstallId, AppId)>,
   pub installs: Vec<PrunedInstall>,
}

#[derive(Debug)]
pub struct PrunedInstall {
   pub install_id:                 InstallId,
   pub unified_push_registrations: usize,
   pub outbox_messages:            usize,
}

pub struct Database {
   conn: Connection,
}

const REGISTRATION_COLUMNS: &str = "install_id, app_id, secret_hash, fcm_token, firebase_app_id,
        firebase_project_id, firebase_api_key, cert_sha1, app_version,
        app_version_name, target_sdk";

fn registration_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Registration> {
   Ok(Registration {
      install_id:          row.get(0)?,
      app_id:              row.get(1)?,
      secret_hash:         row.get(2)?,
      fcm_token:           row.get(3)?,
      firebase_app_id:     row.get(4)?,
      firebase_project_id: row.get(5)?,
      firebase_api_key:    row.get(6)?,
      cert_sha1:           row.get(7)?,
      app_version:         row.get(8)?,
      app_version_name:    row.get(9)?,
      target_sdk:          row.get(10)?,
   })
}

fn count(
   tx: &rusqlite::Transaction<'_>,
   sql: &str,
   params: &[&dyn rusqlite::ToSql],
) -> rusqlite::Result<usize> {
   tx.query_row(sql, params, |row| row.get::<_, i64>(0))
      .map(|count| count as usize)
}

/// Evicts the install's oldest to make room. Refusing the *new* message
/// instead would let an attacker pin a victim's queue at the ceiling forever.
fn trim_outbox(tx: &rusqlite::Transaction<'_>, install_id: &str) -> rusqlite::Result<()> {
   let queued = count(tx, "SELECT COUNT(*) FROM outbox WHERE install_id = ?1", &[
      &install_id,
   ])?;
   if queued < MAX_OUTBOX_ROWS_PER_INSTALL {
      return Ok(());
   }
   let evicted = tx.execute(
      "DELETE FROM outbox WHERE id IN (
             SELECT id FROM outbox WHERE install_id = ?1
             ORDER BY id LIMIT ?2
         )",
      params![
         install_id,
         (queued - MAX_OUTBOX_ROWS_PER_INSTALL + 1) as i64
      ],
   )?;
   tracing::warn!(
      "Outbox for {install_id} hit the {MAX_OUTBOX_ROWS_PER_INSTALL} row ceiling, dropped \
       {evicted} of its oldest messages"
   );
   Ok(())
}

const CREATE_TABLES: &str = include_str!("../sql/schema.sql");

fn column_exists(conn: &rusqlite::Connection, table: &str, column: &str) -> rusqlite::Result<bool> {
   let mut stmt = conn.prepare(&format!("PRAGMA table_info({table})"))?;
   let mut rows = stmt.query([])?;
   while let Some(row) = rows.next()? {
      if row.get::<_, String>(1)? == column {
         return Ok(true);
      }
   }
   Ok(false)
}

fn init_schema(conn: &mut rusqlite::Connection) -> rusqlite::Result<()> {
   let version = conn.query_row("PRAGMA user_version", [], |row| row.get::<_, i32>(0))?;
   if version >= SCHEMA_VERSION {
      return Ok(());
   }

   let tx = conn.transaction()?;

   tx.execute_batch(CREATE_TABLES)?;
   tx.execute(
      "INSERT OR IGNORE INTO installations (install_id, secret_hash)
         SELECT install_id, secret_hash FROM registrations",
      [],
   )?;
   if !column_exists(&tx, "unified_push_registrations", "vapid_pubkey")? {
      tx.execute(
         "ALTER TABLE unified_push_registrations ADD COLUMN vapid_pubkey TEXT",
         [],
      )?;
   }

   // The shim columns are NOT NULL with no default, so they have to go rather
   // than just stop being written. SQLite refuses to drop an indexed column.
   if column_exists(&tx, "outbox", "transport")? {
      tx.execute_batch(
         "DROP INDEX IF EXISTS outbox_up_due;
          DROP INDEX IF EXISTS outbox_socket_order;
          ALTER TABLE outbox DROP COLUMN transport;
          CREATE INDEX outbox_socket_order ON outbox(install_id, id);",
      )?;
   }
   if column_exists(&tx, "registrations", "transport")? {
      tx.execute("ALTER TABLE registrations DROP COLUMN transport", [])?;
   }
   if column_exists(&tx, "registrations", "endpoint")? {
      tx.execute("ALTER TABLE registrations DROP COLUMN endpoint", [])?;
   }
   if !column_exists(&tx, "registrations", "generation")? {
      tx.execute("ALTER TABLE registrations ADD COLUMN generation TEXT", [])?;
   }

   tx.pragma_update(None, "user_version", SCHEMA_VERSION)?;
   tx.commit()
}

impl Database {
   pub async fn new(path: impl AsRef<Path>) -> Result<Self> {
      let conn = Connection::open(path)
         .await
         .context("Failed to open database")?;
      conn
         .call(|conn| -> rusqlite::Result<_> { init_schema(conn) })
         .await
         .context("Failed to initialize database schema")?;
      Ok(Self { conn })
   }

   /// [`Claim`] separates a brand-new `install_id` from a returning one so only
   /// enrollment is rate limited.
   pub async fn claim_installation(
      &self,
      install_id: &InstallId,
      secret_hash: &str,
      install_secret: &InstallSecret,
   ) -> Result<Claim> {
      let install_id = install_id.as_ref().to_owned();
      let secret_hash = secret_hash.to_owned();
      let install_secret = install_secret.expose().to_owned();
      let claim = self
         .conn
         .call(move |conn| -> rusqlite::Result<_> {
            let tx = conn.transaction()?;
            let known = count(
               &tx,
               "SELECT COUNT(*) FROM installations WHERE install_id = ?1",
               &[&install_id],
            )? > 0;
            if !known
               && count(&tx, "SELECT COUNT(*) FROM installations", &[])? >= MAX_TOTAL_INSTALLATIONS
            {
               return Ok(Claim::Refused(QuotaRejection::Server));
            }
            let changed = tx.execute(
               "INSERT INTO installations (install_id, secret_hash, install_secret)
                     VALUES (?1, ?2, ?3)
                     ON CONFLICT(install_id) DO UPDATE SET
                        install_secret = excluded.install_secret,
                        updated_at = CURRENT_TIMESTAMP
                     WHERE installations.secret_hash = excluded.secret_hash",
               params![install_id, secret_hash, install_secret],
            )?;
            tx.commit()?;
            Ok(match (known, changed > 0) {
               (false, _) => Claim::Enrolled,
               (true, true) => Claim::Existing,
               (true, false) => Claim::Denied,
            })
         })
         .await
         .context("Failed to claim installation")?;
      Ok(claim)
   }

   /// Backs the `/up/` ceiling, so unauthenticated senders cannot fill the
   /// volume out from under queued messages.
   pub async fn size_bytes(&self) -> Result<u64> {
      let size = self
         .conn
         .call(|conn| -> rusqlite::Result<_> {
            let pages = conn.query_row("PRAGMA page_count", [], |row| row.get::<_, i64>(0))?;
            let page_size = conn.query_row("PRAGMA page_size", [], |row| row.get::<_, i64>(0))?;
            Ok(pages.max(0) as u64 * page_size.max(0) as u64)
         })
         .await
         .context("Failed to measure database size")?;
      Ok(size)
   }

   pub async fn verify_installation(
      &self,
      install_id: &str,
      secret: &InstallSecret,
   ) -> Result<bool> {
      let install_id = install_id.to_owned();
      let queried_install_id = install_id.clone();
      let candidate_secret = secret.expose().to_owned();
      let stored_hash = self
         .conn
         .call(move |conn| -> rusqlite::Result<_> {
            let result = conn.query_row(
               "SELECT secret_hash FROM installations WHERE install_id = ?1",
               [queried_install_id],
               |row| row.get::<_, String>(0),
            );
            match result {
               Ok(hash) => Ok(Some(hash)),
               Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
               Err(error) => Err(error),
            }
         })
         .await
         .context("Failed to verify installation")?;
      let verified = verify_secret(secret, stored_hash.as_deref().unwrap_or(""));
      if verified {
         let install_id = install_id.clone();
         self
            .conn
            .call(move |conn| -> rusqlite::Result<_> {
               conn.execute(
                  "UPDATE installations
                         SET install_secret = ?2, updated_at = CURRENT_TIMESTAMP
                         WHERE install_id = ?1",
                  params![install_id, candidate_secret],
               )?;
               Ok(())
            })
            .await
            .context("Failed to refresh installation secret")?;
      }
      Ok(verified)
   }

   /// Undoes one specific write. `generation` is what makes that precise, so a
   /// concurrent registration for the same app that has already succeeded is
   /// left alone rather than deleted out from under its 200 response. Returns
   /// false when the row now belongs to someone else.
   ///
   /// Removing the install is unconditional because the predicate spares any
   /// install that still owns something, and sharing the transaction stops a
   /// concurrent rollback from seeing the row this one is retiring.
   pub async fn roll_back_registration(
      &self,
      install_id: &InstallId,
      app_id: &AppId,
      generation: &str,
   ) -> Result<bool> {
      let install_id = install_id.as_ref().to_owned();
      let app_id = app_id.as_ref().to_owned();
      let generation = generation.to_owned();
      let kind = MessageKind::Fcm.as_ref().to_owned();
      let rolled_back = self
         .conn
         .call(move |conn| -> rusqlite::Result<_> {
            let tx = conn.transaction()?;
            let owned = tx.execute(
               "DELETE FROM registrations
                     WHERE install_id = ?1 AND app_id = ?2 AND generation = ?3",
               params![install_id, app_id, generation],
            )? > 0;
            if !owned {
               tx.commit()?;
               return Ok(false);
            }
            // fcm_sessions deliberately survives. It holds the FCM token the
            // app's server already knows, and re-registering would mint a new
            // one the server never learns.
            tx.execute(
               "DELETE FROM acked_messages WHERE install_id = ?1 AND app_id = ?2",
               params![install_id, app_id],
            )?;
            // Scoped to FCM. The same (install, app) pair can hold queued
            // UnifiedPush messages, which this registration never owned.
            tx.execute(
               "DELETE FROM outbox
                     WHERE install_id = ?1 AND app_id = ?2 AND kind = ?3",
               params![install_id, app_id, kind],
            )?;
            tx.execute(
               "DELETE FROM installations
                     WHERE install_id = ?1
                       AND NOT EXISTS (
                           SELECT 1 FROM registrations WHERE install_id = ?1
                       )
                       AND NOT EXISTS (
                           SELECT 1 FROM unified_push_registrations WHERE install_id = ?1
                       )",
               [&install_id],
            )?;
            tx.commit()?;
            Ok(true)
         })
         .await
         .context("Failed to roll back registration")?;
      Ok(rolled_back)
   }

   #[cfg(test)]
   pub(crate) async fn delete_unused_installation(&self, install_id: &InstallId) -> Result<bool> {
      let install_id = install_id.as_ref().to_owned();
      let deleted = self
         .conn
         .call(move |conn| -> rusqlite::Result<_> {
            let changed = conn.execute(
               "DELETE FROM installations
                     WHERE install_id = ?1
                       AND NOT EXISTS (
                           SELECT 1 FROM registrations WHERE install_id = ?1
                       )
                       AND NOT EXISTS (
                           SELECT 1 FROM unified_push_registrations WHERE install_id = ?1
                       )",
               [install_id],
            )?;
            Ok(changed > 0)
         })
         .await
         .context("Failed to delete unused installation")?;
      Ok(deleted)
   }

   /// `None` when the install does not exist, which is how a caller tells a
   /// first registration from a returning one without writing anything.
   pub async fn installation_hash(&self, install_id: &InstallId) -> Result<Option<String>> {
      let install_id = install_id.as_ref().to_owned();
      let hash = self
         .conn
         .call(move |conn| -> rusqlite::Result<_> {
            match conn.query_row(
               "SELECT secret_hash FROM installations WHERE install_id = ?1",
               [install_id],
               |row| row.get::<_, String>(0),
            ) {
               Ok(hash) => Ok(Some(hash)),
               Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
               Err(error) => Err(error),
            }
         })
         .await
         .context("Failed to look up installation")?;
      Ok(hash)
   }

   pub async fn installation_secret(
      &self,
      install_id: &InstallId,
   ) -> Result<Option<InstallSecret>> {
      let install_id = install_id.as_ref().to_owned();
      let secret = self
         .conn
         .call(move |conn| -> rusqlite::Result<_> {
            let result = conn.query_row(
               "SELECT install_secret FROM installations WHERE install_id = ?1",
               [install_id],
               |row| row.get::<_, Option<String>>(0),
            );
            match result {
               Ok(secret) => Ok(secret),
               Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
               Err(error) => Err(error),
            }
         })
         .await
         .context("Failed to load installation secret")?;
      Ok(secret.map(InstallSecret::from))
   }

   pub async fn touch_installation(&self, install_id: &InstallId) -> Result<()> {
      let install_id = install_id.clone();
      self
         .conn
         .call(move |conn| -> rusqlite::Result<_> {
            conn.execute(
               "UPDATE installations SET updated_at = CURRENT_TIMESTAMP
                     WHERE install_id = ?1",
               [install_id],
            )?;
            Ok(())
         })
         .await
         .context("Failed to touch installation")?;
      Ok(())
   }

   pub async fn register_unified_push(
      &self,
      install_id: &InstallId,
      app_id: &AppId,
      connector_token: &ConnectorToken,
      endpoint_token: &str,
      vapid_pubkey: Option<&str>,
   ) -> Result<UnifiedPushClaim> {
      let install_id = install_id.as_ref().to_owned();
      let app_id = app_id.as_ref().to_owned();
      let connector_token = connector_token.as_ref().to_owned();
      let endpoint_token = endpoint_token.to_owned();
      let vapid_pubkey = vapid_pubkey.map(str::to_owned);
      let result = self
         .conn
         .call(move |conn| -> rusqlite::Result<_> {
            let tx = conn.transaction()?;
            let existing = tx.query_row(
               "SELECT app_id, endpoint_token
                     FROM unified_push_registrations
                     WHERE install_id = ?1 AND connector_token = ?2",
               params![install_id, connector_token],
               |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
            );
            let resolved = match existing {
               Ok((owner, endpoint)) if owner == app_id => {
                  tx.execute(
                     "UPDATE unified_push_registrations
                             SET updated_at = CURRENT_TIMESTAMP,
                                 vapid_pubkey = ?3
                             WHERE install_id = ?1 AND connector_token = ?2",
                     params![install_id, connector_token, vapid_pubkey],
                  )?;
                  UnifiedPushClaim::Registered(endpoint)
               },
               Ok(_) => UnifiedPushClaim::TokenOwnedByAnotherApp,
               Err(rusqlite::Error::QueryReturnedNoRows) => {
                  if count(
                     &tx,
                     "SELECT COUNT(*) FROM unified_push_registrations WHERE install_id = ?1",
                     &[&install_id],
                  )? >= MAX_UNIFIED_PUSH_PER_INSTALL
                  {
                     return Ok(UnifiedPushClaim::Refused(QuotaRejection::Install));
                  }
                  tx.execute(
                     "INSERT INTO unified_push_registrations
                             (install_id, app_id, connector_token, endpoint_token, vapid_pubkey)
                             VALUES (?1, ?2, ?3, ?4, ?5)",
                     params![
                        install_id,
                        app_id,
                        connector_token,
                        endpoint_token,
                        vapid_pubkey
                     ],
                  )?;
                  UnifiedPushClaim::Registered(endpoint_token)
               },
               Err(error) => return Err(error),
            };
            tx.commit()?;
            Ok(resolved)
         })
         .await
         .context("Failed to register UnifiedPush connector")?;
      Ok(result)
   }

   pub async fn unregister_unified_push(
      &self,
      install_id: &InstallId,
      app_id: &AppId,
      connector_token: &ConnectorToken,
   ) -> Result<bool> {
      let install_id = install_id.as_ref().to_owned();
      let app_id = app_id.as_ref().to_owned();
      let connector_token = connector_token.as_ref().to_owned();
      let deleted = self
         .conn
         .call(move |conn| -> rusqlite::Result<_> {
            let tx = conn.transaction()?;
            let changed = tx.execute(
               "DELETE FROM unified_push_registrations
                     WHERE install_id = ?1 AND app_id = ?2 AND connector_token = ?3",
               params![install_id, app_id, connector_token],
            )?;
            if changed > 0 {
               tx.execute(
                  "DELETE FROM outbox
                         WHERE install_id = ?1 AND app_id = ?2
                           AND connector_token = ?3",
                  params![install_id, app_id, connector_token],
               )?;
            }
            tx.commit()?;
            Ok(changed > 0)
         })
         .await
         .context("Failed to unregister UnifiedPush connector")?;
      Ok(deleted)
   }

   pub async fn delete_stale_unified_push_registrations(
      &self,
      install_id: &InstallId,
      retained_tokens: &[ConnectorToken],
   ) -> Result<usize> {
      let install_id = install_id.as_ref().to_owned();
      let retained_tokens = retained_tokens
         .iter()
         .map(|token| token.as_ref().to_owned())
         .collect::<HashSet<_>>();
      let deleted = self
         .conn
         .call(move |conn| -> rusqlite::Result<_> {
            let tx = conn.transaction()?;
            let stored_tokens = {
               let mut stmt = tx.prepare(
                  "SELECT connector_token FROM unified_push_registrations
                        WHERE install_id = ?1",
               )?;
               let rows = stmt.query_map([&install_id], |row| row.get::<_, String>(0))?;
               rows.collect::<rusqlite::Result<Vec<_>>>()?
            };
            let stale_tokens = stored_tokens
               .into_iter()
               .filter(|token| !retained_tokens.contains(token));
            let mut deleted = 0;
            for connector_token in stale_tokens {
               tx.execute(
                  "DELETE FROM outbox
                        WHERE install_id = ?1 AND connector_token = ?2",
                  params![install_id, connector_token],
               )?;
               deleted += tx.execute(
                  "DELETE FROM unified_push_registrations
                        WHERE install_id = ?1 AND connector_token = ?2",
                  params![install_id, connector_token],
               )?;
            }
            tx.commit()?;
            Ok(deleted)
         })
         .await
         .context("Failed to delete stale UnifiedPush registrations")?;
      Ok(deleted)
   }

   pub async fn get_unified_push_endpoint(
      &self,
      endpoint_token: &str,
   ) -> Result<Option<UnifiedPushRegistration>> {
      let endpoint_token = endpoint_token.to_owned();
      let registration = self
         .conn
         .call(move |conn| -> rusqlite::Result<_> {
            let result = conn.query_row(
               "SELECT install_id, app_id, connector_token, vapid_pubkey
                     FROM unified_push_registrations
                     WHERE endpoint_token = ?1",
               [endpoint_token],
               |row| {
                  Ok(UnifiedPushRegistration {
                     install_id:      row.get(0)?,
                     app_id:          row.get(1)?,
                     connector_token: row.get(2)?,
                     vapid_pubkey:    row.get(3)?,
                  })
               },
            );
            match result {
               Ok(registration) => Ok(Some(registration)),
               Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
               Err(error) => Err(error),
            }
         })
         .await
         .context("Failed to load UnifiedPush endpoint")?;
      Ok(registration)
   }

   pub async fn enqueue_fcm_message(
      &self,
      install_id: &InstallId,
      app_id: &AppId,
      persistent_id: Option<&str>,
      payload: &[u8],
   ) -> Result<Option<MessageId>> {
      let install_id = install_id.as_ref().to_owned();
      let app_id = app_id.as_ref().to_owned();
      let kind = MessageKind::Fcm.as_ref().to_owned();
      let persistent_id = persistent_id.map(str::to_owned);
      let payload = payload.to_vec();
      let message_id = self
         .conn
         .call(move |conn| -> rusqlite::Result<_> {
            let tx = conn.transaction()?;
            trim_outbox(&tx, &install_id)?;
            let was_acked = if let Some(pid) = persistent_id.as_deref() {
               tx.query_row(
                  "SELECT EXISTS(
                            SELECT 1 FROM acked_messages
                            WHERE install_id = ?1 AND app_id = ?2 AND persistent_id = ?3
                         )",
                  params![install_id, app_id, pid],
                  |row| row.get::<_, bool>(0),
               )?
            } else {
               false
            };

            let message_id = if !was_acked && !payload.is_empty() {
               tx.execute(
                  "INSERT OR IGNORE INTO outbox
                         (install_id, app_id, kind, persistent_id, payload)
                         VALUES (?1, ?2, ?3, ?4, ?5)",
                  params![install_id, app_id, kind, persistent_id, payload],
               )?;
               Some(tx.query_row(
                  "SELECT id FROM outbox
                         WHERE install_id = ?1 AND app_id = ?2
                           AND (
                               (?3 IS NOT NULL AND persistent_id = ?3)
                               OR (?3 IS NULL AND id = last_insert_rowid())
                           )
                         ORDER BY id DESC LIMIT 1",
                  params![install_id, app_id, persistent_id],
                  |row| Ok(MessageId::new(row.get(0)?)),
               )?)
            } else {
               None
            };

            if let Some(pid) = persistent_id {
               tx.execute(
                  "INSERT OR IGNORE INTO acked_messages
                         (install_id, app_id, persistent_id)
                         VALUES (?1, ?2, ?3)",
                  params![install_id, app_id, pid],
               )?;
               tx.execute(
                  "DELETE FROM acked_messages
                         WHERE install_id = ?1 AND app_id = ?2
                           AND rowid NOT IN (
                               SELECT rowid FROM acked_messages
                               WHERE install_id = ?1 AND app_id = ?2
                               ORDER BY rowid DESC LIMIT 500
                           )",
                  params![install_id, app_id],
               )?;
            }
            tx.commit()?;
            Ok(message_id)
         })
         .await
         .context("Failed to enqueue FCM message")?;
      Ok(message_id)
   }

   pub async fn enqueue_unified_push_message(
      &self,
      install_id: &InstallId,
      app_id: &AppId,
      connector_token: &ConnectorToken,
      payload: &[u8],
   ) -> Result<MessageId> {
      let install_id = install_id.as_ref().to_owned();
      let app_id = app_id.as_ref().to_owned();
      let connector_token = connector_token.as_ref().to_owned();
      let kind = MessageKind::UnifiedPush.as_ref().to_owned();
      let payload = payload.to_vec();
      let id = self
         .conn
         .call(move |conn| -> rusqlite::Result<_> {
            let tx = conn.transaction()?;
            trim_outbox(&tx, &install_id)?;
            tx.execute(
               "INSERT INTO outbox
                     (install_id, app_id, kind, connector_token, payload)
                     VALUES (?1, ?2, ?3, ?4, ?5)",
               params![install_id, app_id, kind, connector_token, payload],
            )?;
            let id = MessageId::new(tx.last_insert_rowid());
            tx.commit()?;
            Ok(id)
         })
         .await
         .context("Failed to enqueue UnifiedPush message")?;
      Ok(id)
   }

   /// The head message is always selected by id alone. A head that was
   /// deferred after a failed delivery returns `None` instead of being
   /// skipped, because the client cursor is monotonic and a message behind
   /// an acked id becomes unreachable.
   pub async fn next_socket_message(
      &self,
      install_id: &InstallId,
      cursor: Cursor,
   ) -> Result<Option<OutboxMessage>> {
      let install_id = install_id.clone();
      let message = self
         .conn
         .call(move |conn| -> rusqlite::Result<_> {
            let result = conn.query_row(
               "SELECT id, app_id, kind, connector_token, payload, attempts,
                            next_attempt_at <= CURRENT_TIMESTAMP
                     FROM outbox
                     WHERE install_id = ?1 AND id > ?2
                     ORDER BY id LIMIT 1",
               params![install_id, cursor],
               |row| {
                  Ok((
                     OutboxMessage {
                        id:              row.get(0)?,
                        app_id:          row.get(1)?,
                        kind:            row.get(2)?,
                        connector_token: row.get(3)?,
                        payload:         row.get(4)?,
                        attempts:        row.get(5)?,
                     },
                     row.get::<_, bool>(6)?,
                  ))
               },
            );
            match result {
               Ok((message, true)) => Ok(Some(message)),
               Ok((_, false)) | Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
               Err(error) => Err(error),
            }
         })
         .await
         .context("Failed to load socket outbox")?;
      Ok(message)
   }

   pub async fn ack_socket_message(&self, install_id: &InstallId, id: MessageId) -> Result<bool> {
      let install_id = install_id.clone();
      let deleted = self
         .conn
         .call(move |conn| -> rusqlite::Result<_> {
            let changed = conn.execute(
               "DELETE FROM outbox WHERE id = ?1 AND install_id = ?2",
               params![id, install_id],
            )?;
            Ok(changed == 1)
         })
         .await
         .context("Failed to acknowledge socket message")?;
      Ok(deleted)
   }

   pub async fn ack_socket_through(&self, install_id: &InstallId, cursor: Cursor) -> Result<usize> {
      let install_id = install_id.clone();
      let deleted = self
         .conn
         .call(move |conn| -> rusqlite::Result<_> {
            let changed = conn.execute(
               "DELETE FROM outbox WHERE install_id = ?1 AND id <= ?2",
               params![install_id, cursor],
            )?;
            Ok(changed)
         })
         .await
         .context("Failed to apply socket resume cursor")?;
      Ok(deleted)
   }

   pub async fn delete_outbox_message(&self, id: MessageId) -> Result<()> {
      self
         .conn
         .call(move |conn| -> rusqlite::Result<_> {
            conn.execute("DELETE FROM outbox WHERE id = ?1", [id])?;
            Ok(())
         })
         .await
         .context("Failed to delete outbox message")?;
      Ok(())
   }

   pub async fn defer_outbox_message(&self, id: MessageId, seconds: i64) -> Result<()> {
      self
         .conn
         .call(move |conn| -> rusqlite::Result<_> {
            conn.execute(
               "UPDATE outbox
                     SET attempts = attempts + 1,
                         next_attempt_at = datetime('now', ?2)
                     WHERE id = ?1",
               params![id, format!("+{seconds} seconds")],
            )?;
            Ok(())
         })
         .await
         .context("Failed to defer outbox message")?;
      Ok(())
   }

   #[cfg(test)]
   pub(crate) async fn outbox_len(&self, install_id: &InstallId) -> Result<usize> {
      let install_id = install_id.as_ref().to_owned();
      let queued = self
         .conn
         .call(move |conn| -> rusqlite::Result<_> {
            conn
               .query_row(
                  "SELECT COUNT(*) FROM outbox WHERE install_id = ?1",
                  [install_id],
                  |row| row.get::<_, i64>(0),
               )
               .map(|count| count as usize)
         })
         .await
         .context("Failed to count outbox rows")?;
      Ok(queued)
   }

   /// One transaction, so reaching a thousands-of-rows ceiling stays fast.
   #[cfg(test)]
   pub(crate) async fn seed_outbox(
      &self,
      install_id: &InstallId,
      app_id: &AppId,
      rows: usize,
   ) -> Result<()> {
      let install_id = install_id.as_ref().to_owned();
      let app_id = app_id.as_ref().to_owned();
      let kind = MessageKind::Fcm.as_ref().to_owned();
      self
         .conn
         .call(move |conn| -> rusqlite::Result<_> {
            let tx = conn.transaction()?;
            {
               let mut stmt = tx.prepare(
                  "INSERT INTO outbox (install_id, app_id, kind, payload)
                        VALUES (?1, ?2, ?3, ?4)",
               )?;
               for row in 0..rows {
                  stmt.execute(params![
                     install_id,
                     app_id,
                     kind,
                     format!("{row}").as_bytes()
                  ])?;
               }
            }
            tx.commit()?;
            Ok(())
         })
         .await
         .context("Failed to seed outbox")?;
      Ok(())
   }

   #[cfg(test)]
   pub(crate) async fn make_outbox_message_due(&self, id: MessageId) -> Result<()> {
      self
         .conn
         .call(move |conn| -> rusqlite::Result<_> {
            conn.execute(
               "UPDATE outbox
                     SET next_attempt_at = datetime('now', '-1 seconds')
                     WHERE id = ?1",
               [id],
            )?;
            Ok(())
         })
         .await
         .context("Failed to force outbox message due")?;
      Ok(())
   }

   pub async fn max_outbox_id(&self) -> Result<MessageId> {
      let id = self
         .conn
         .call(|conn| -> rusqlite::Result<_> {
            conn.query_row("SELECT COALESCE(MAX(id), 0) FROM outbox", [], |row| {
               row.get::<_, MessageId>(0)
            })
         })
         .await
         .context("Failed to load maximum outbox id")?;
      Ok(id)
   }

   /// `Denied` means a concurrent request claimed the pair first. Quotas only
   /// apply to a new pair, so an install at its ceiling can still re-register.
   pub async fn save_registration(&self, reg: &Registration, generation: &str) -> Result<Claim> {
      let reg = reg.clone();
      let generation = generation.to_owned();
      let saved = self
         .conn
         .call(move |conn| -> rusqlite::Result<_> {
            let tx = conn.transaction()?;
            let install_id = reg.install_id.as_ref();
            let known = count(
               &tx,
               "SELECT COUNT(*) FROM registrations WHERE install_id = ?1 AND app_id = ?2",
               &[&install_id, &reg.app_id.as_ref()],
            )? > 0;
            if !known {
               if count(
                  &tx,
                  "SELECT COUNT(*) FROM registrations WHERE install_id = ?1",
                  &[&install_id],
               )? >= MAX_REGISTRATIONS_PER_INSTALL
               {
                  return Ok(Claim::Refused(QuotaRejection::Install));
               }
               if count(&tx, "SELECT COUNT(*) FROM registrations", &[])? >= MAX_TOTAL_REGISTRATIONS
               {
                  return Ok(Claim::Refused(QuotaRejection::Server));
               }
            }
            let changed = tx.execute(
               "INSERT INTO registrations
                     (install_id, app_id, secret_hash, fcm_token, firebase_app_id,
                      firebase_project_id, firebase_api_key, cert_sha1, app_version,
                      app_version_name, target_sdk, generation)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
                     ON CONFLICT(install_id, app_id) DO UPDATE SET
                        secret_hash = excluded.secret_hash,
                        fcm_token = excluded.fcm_token,
                        firebase_app_id = excluded.firebase_app_id,
                        firebase_project_id = excluded.firebase_project_id,
                        firebase_api_key = excluded.firebase_api_key,
                        cert_sha1 = excluded.cert_sha1,
                        app_version = excluded.app_version,
                        app_version_name = excluded.app_version_name,
                        target_sdk = excluded.target_sdk,
                        generation = excluded.generation,
                        updated_at = CURRENT_TIMESTAMP
                     WHERE registrations.secret_hash = excluded.secret_hash",
               params![
                  install_id,
                  reg.app_id.as_ref(),
                  reg.secret_hash,
                  reg.fcm_token,
                  reg.firebase_app_id,
                  reg.firebase_project_id,
                  reg.firebase_api_key,
                  reg.cert_sha1,
                  reg.app_version,
                  reg.app_version_name,
                  reg.target_sdk,
                  generation,
               ],
            )?;
            tx.commit()?;
            Ok(match (known, changed > 0) {
               (false, _) => Claim::Enrolled,
               (true, true) => Claim::Existing,
               (true, false) => Claim::Denied,
            })
         })
         .await
         .context("Failed to save registration")?;
      Ok(saved)
   }

   pub async fn get_registration(
      &self,
      install_id: &InstallId,
      app_id: &AppId,
   ) -> Result<Option<Registration>> {
      let install_id = install_id.clone();
      let app_id = app_id.clone();
      let result = self
         .conn
         .call(move |conn| -> rusqlite::Result<_> {
            let result = conn.query_row(
               &format!(
                  "SELECT {REGISTRATION_COLUMNS} FROM registrations
                        WHERE install_id = ?1 AND app_id = ?2"
               ),
               params![install_id, app_id],
               registration_from_row,
            );
            match result {
               Ok(registration) => Ok(Some(registration)),
               Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
               Err(error) => Err(error),
            }
         })
         .await
         .context("Failed to get registration")?;
      Ok(result)
   }

   pub async fn delete_registration(&self, install_id: &InstallId, app_id: &AppId) -> Result<()> {
      let install_id = install_id.as_ref().to_owned();
      let app_id = app_id.as_ref().to_owned();
      let kind = MessageKind::Fcm.as_ref().to_owned();
      self
         .conn
         .call(move |conn| -> rusqlite::Result<_> {
            for table in ["registrations", "fcm_sessions", "acked_messages"] {
               conn.execute(
                  &format!("DELETE FROM {table} WHERE install_id = ?1 AND app_id = ?2"),
                  params![install_id, app_id],
               )?;
            }
            conn.execute(
               "DELETE FROM outbox
                     WHERE install_id = ?1 AND app_id = ?2 AND kind = ?3",
               params![install_id, app_id, kind],
            )?;
            Ok(())
         })
         .await
         .context("Failed to delete registration")?;
      Ok(())
   }

   pub async fn list_registrations(&self) -> Result<Vec<Registration>> {
      let result = self
         .conn
         .call(|conn| -> rusqlite::Result<_> {
            let mut stmt =
               conn.prepare(&format!("SELECT {REGISTRATION_COLUMNS} FROM registrations"))?;
            let rows = stmt.query_map([], registration_from_row)?;
            rows.collect()
         })
         .await
         .context("Failed to list registrations")?;
      Ok(result)
   }

   pub async fn count_registrations(&self) -> Result<usize> {
      let count = self
         .conn
         .call(|conn| -> rusqlite::Result<_> {
            let count = conn.query_row("SELECT COUNT(*) FROM registrations", [], |row| {
               row.get::<_, i64>(0)
            })?;
            Ok(count as usize)
         })
         .await
         .context("Failed to count registrations")?;
      Ok(count)
   }

   /// Reap rows whose client has stopped heartbeating (app data cleared, app
   /// uninstalled). Every row has a secret and therefore a client that
   /// re-registers daily, so silence past the cutoff means the install is
   /// genuinely gone. Installs that stopped attaching entirely lose their
   /// `UnifiedPush` registrations and queued messages too, so `/up/` stops
   /// accepting deliveries no device can ever drain.
   pub async fn prune_stale(&self, max_age_days: u32) -> Result<PruneOutcome> {
      let pruned = self
         .conn
         .call(move |conn| -> rusqlite::Result<_> {
            let cutoff = format!("-{max_age_days} days");
            let mut stmt = conn.prepare(
               "SELECT install_id, app_id FROM registrations
                     WHERE updated_at < datetime('now', ?1)",
            )?;
            let pairs = stmt
               .query_map([&cutoff], |row| {
                  Ok((row.get::<_, InstallId>(0)?, row.get::<_, AppId>(1)?))
               })?
               .collect::<rusqlite::Result<Vec<_>>>()?;
            let kind = MessageKind::Fcm.as_ref();
            for (install_id, app_id) in &pairs {
               for table in ["registrations", "fcm_sessions", "acked_messages"] {
                  conn.execute(
                     &format!("DELETE FROM {table} WHERE install_id = ?1 AND app_id = ?2"),
                     params![install_id.as_ref(), app_id.as_ref()],
                  )?;
               }
               conn.execute(
                  "DELETE FROM outbox
                         WHERE install_id = ?1 AND app_id = ?2 AND kind = ?3",
                  params![install_id.as_ref(), app_id.as_ref(), kind],
               )?;
            }

            // An install with no registrations can keep updated_at fresh just by
            // re-authenticating, so the second clause retires one that never
            // registered anything on its age instead.
            let stale_installs = {
               let unused_cutoff = format!("-{UNUSED_INSTALL_MAX_AGE_DAYS} days");
               let mut stmt = conn.prepare(
                  "SELECT install_id FROM installations
                        WHERE NOT EXISTS (
                              SELECT 1 FROM registrations
                              WHERE registrations.install_id = installations.install_id
                          )
                          AND (updated_at < datetime('now', ?1)
                               OR created_at < datetime('now', ?2))",
               )?;
               stmt
                  .query_map(params![&cutoff, &unused_cutoff], |row| {
                     row.get::<_, InstallId>(0)
                  })?
                  .collect::<rusqlite::Result<Vec<_>>>()?
            };
            let mut installs = Vec::with_capacity(stale_installs.len());
            for install_id in stale_installs {
               let unified_push_registrations = conn.execute(
                  "DELETE FROM unified_push_registrations WHERE install_id = ?1",
                  [install_id.as_ref()],
               )?;
               let outbox_messages = conn.execute("DELETE FROM outbox WHERE install_id = ?1", [
                  install_id.as_ref(),
               ])?;
               conn.execute("DELETE FROM installations WHERE install_id = ?1", [
                  install_id.as_ref(),
               ])?;
               installs.push(PrunedInstall {
                  install_id,
                  unified_push_registrations,
                  outbox_messages,
               });
            }
            Ok(PruneOutcome { pairs, installs })
         })
         .await
         .context("Failed to prune stale registrations")?;
      Ok(pruned)
   }

   #[cfg(test)]
   pub(crate) async fn backdate_installation(
      &self,
      install_id: &InstallId,
      days: u32,
   ) -> Result<()> {
      self
         .backdate_installation_column(install_id, "updated_at", days)
         .await
   }

   #[cfg(test)]
   pub(crate) async fn backdate_installation_column(
      &self,
      install_id: &InstallId,
      column: &'static str,
      days: u32,
   ) -> Result<()> {
      let install_id = install_id.as_ref().to_owned();
      self
         .conn
         .call(move |conn| -> rusqlite::Result<_> {
            conn.execute(
               &format!(
                  "UPDATE installations SET {column} = datetime('now', ?2)
                        WHERE install_id = ?1"
               ),
               params![install_id, format!("-{days} days")],
            )?;
            Ok(())
         })
         .await
         .context("Failed to backdate installation")?;
      Ok(())
   }

   /// Most recent acks first; the list is capped because MCS login carries it
   /// inline.
   pub async fn recent_acks(
      &self,
      install_id: &InstallId,
      app_id: &AppId,
      limit: usize,
   ) -> Result<Vec<String>> {
      let install_id = install_id.as_ref().to_owned();
      let app_id = app_id.as_ref().to_owned();
      let result = self
         .conn
         .call(move |conn| -> rusqlite::Result<_> {
            let mut stmt = conn.prepare(
               "SELECT persistent_id FROM acked_messages
                     WHERE install_id = ?1 AND app_id = ?2
                     ORDER BY rowid DESC LIMIT ?3",
            )?;
            let rows =
               stmt.query_map(params![install_id, app_id, limit as i64], |row| row.get(0))?;
            let mut ids = Vec::new();
            for row in rows {
               ids.push(row?);
            }
            Ok(ids)
         })
         .await
         .context("Failed to load acks")?;
      Ok(result)
   }

   pub async fn save_fcm_session(
      &self,
      install_id: &InstallId,
      app_id: &AppId,
      data: &str,
   ) -> Result<()> {
      let install_id = install_id.as_ref().to_owned();
      let app_id = app_id.as_ref().to_owned();
      let data = data.to_owned();
      self
         .conn
         .call(move |conn| -> rusqlite::Result<_> {
            conn.execute(
               "INSERT OR REPLACE INTO fcm_sessions (install_id, app_id, registration_data)
                     VALUES (?1, ?2, ?3)",
               params![install_id, app_id, data],
            )?;
            Ok(())
         })
         .await
         .context("Failed to save FCM session")?;
      Ok(())
   }

   pub async fn get_fcm_session(
      &self,
      install_id: &InstallId,
      app_id: &AppId,
   ) -> Result<Option<String>> {
      let install_id = install_id.as_ref().to_owned();
      let app_id = app_id.as_ref().to_owned();
      let result = self
         .conn
         .call(move |conn| -> rusqlite::Result<_> {
            let result = conn.query_row(
               "SELECT registration_data FROM fcm_sessions
                     WHERE install_id = ?1 AND app_id = ?2",
               params![install_id, app_id],
               |row| row.get::<_, String>(0),
            );

            match result {
               Ok(data) => Ok(Some(data)),
               Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
               Err(error) => Err(error),
            }
         })
         .await
         .context("Failed to get FCM session")?;
      Ok(result)
   }
}

#[cfg(test)]
mod tests {
   use std::slice;

   use super::*;

   async fn fresh_db() -> Database {
      Database::new(":memory:").await.unwrap()
   }

   fn registration(install_id: &InstallId, app_id: &AppId) -> Registration {
      Registration {
         install_id:          install_id.clone(),
         app_id:              app_id.clone(),
         secret_hash:         "hash".to_owned(),
         fcm_token:           None,
         firebase_app_id:     "1:123:android:abc".to_owned(),
         firebase_project_id: "proj".to_owned(),
         firebase_api_key:    "key".to_owned(),
         cert_sha1:           None,
         app_version:         None,
         app_version_name:    None,
         target_sdk:          None,
      }
   }

   #[tokio::test]
   async fn unified_push_registration_can_clear_vapid_key() {
      let db = fresh_db().await;
      let install_id = InstallId::try_from("0123456789abcdef").unwrap();
      let app_id = AppId::trusted("com.app");
      let connector_token = ConnectorToken::try_from("connector-1").unwrap();
      let secret = InstallSecret::from("secret");

      assert_eq!(
         db.claim_installation(&install_id, "secret-hash", &secret)
            .await
            .unwrap(),
         Claim::Enrolled
      );
      assert_eq!(
         db.register_unified_push(
            &install_id,
            &app_id,
            &connector_token,
            "endpoint-1",
            Some("vapid-key"),
         )
         .await
         .unwrap(),
         UnifiedPushClaim::Registered("endpoint-1".to_owned())
      );
      assert_eq!(
         db.get_unified_push_endpoint("endpoint-1")
            .await
            .unwrap()
            .unwrap()
            .vapid_pubkey
            .as_deref(),
         Some("vapid-key")
      );

      assert_eq!(
         db.register_unified_push(&install_id, &app_id, &connector_token, "endpoint-2", None,)
            .await
            .unwrap(),
         UnifiedPushClaim::Registered("endpoint-1".to_owned())
      );
      assert_eq!(
         db.get_unified_push_endpoint("endpoint-1")
            .await
            .unwrap()
            .unwrap()
            .vapid_pubkey,
         None
      );
   }

   #[tokio::test]
   async fn unified_push_reconcile_prunes_only_one_installation() {
      let db = fresh_db().await;
      let first_install = InstallId::try_from("0123456789abcdef").unwrap();
      let second_install = InstallId::try_from("fedcba9876543210").unwrap();
      let app_id = AppId::trusted("com.app");
      let keyed_app_id = AppId::trusted("im.molly.app");
      let first_token = ConnectorToken::try_from("first-token").unwrap();
      let stale_token = ConnectorToken::try_from("stale-token").unwrap();
      let keyed_token = ConnectorToken::try_from("keyed-token").unwrap();
      let other_install_token = ConnectorToken::try_from("other-install-token").unwrap();
      let secret = InstallSecret::from("secret");

      assert_eq!(
         db.claim_installation(&first_install, "first-hash", &secret)
            .await
            .unwrap(),
         Claim::Enrolled
      );
      assert_eq!(
         db.claim_installation(&second_install, "second-hash", &secret)
            .await
            .unwrap(),
         Claim::Enrolled
      );
      for (install_id, app_id, connector_token, endpoint_token, vapid) in [
         (
            &first_install,
            &app_id,
            &first_token,
            "first-endpoint",
            None,
         ),
         (
            &first_install,
            &app_id,
            &stale_token,
            "stale-endpoint",
            None,
         ),
         (
            &first_install,
            &keyed_app_id,
            &keyed_token,
            "keyed-endpoint",
            Some("vapid-key"),
         ),
         (
            &second_install,
            &app_id,
            &other_install_token,
            "other-endpoint",
            None,
         ),
      ] {
         db.register_unified_push(install_id, app_id, connector_token, endpoint_token, vapid)
            .await
            .unwrap();
      }

      assert_eq!(
         db.delete_stale_unified_push_registrations(&first_install, &[
            first_token.clone(),
            keyed_token.clone()
         ],)
            .await
            .unwrap(),
         1
      );
      assert!(
         db.get_unified_push_endpoint("stale-endpoint")
            .await
            .unwrap()
            .is_none()
      );
      assert_eq!(
         db.get_unified_push_endpoint("keyed-endpoint")
            .await
            .unwrap()
            .unwrap()
            .vapid_pubkey
            .as_deref(),
         Some("vapid-key")
      );
      assert!(
         db.get_unified_push_endpoint("other-endpoint")
            .await
            .unwrap()
            .is_some()
      );
   }

   /// Two concurrent registrations for the same app: the slow one's rollback
   /// must not delete the row the fast one already answered 200 for.
   #[tokio::test]
   async fn a_rollback_only_removes_its_own_write() {
      let db = fresh_db().await;
      let secret = InstallSecret::from("secret");
      let install_id = InstallId::try_from("0123456789abcdef").unwrap();
      let app_id = AppId::trusted("com.app");

      db.claim_installation(&install_id, "hash", &secret)
         .await
         .unwrap();
      db.save_registration(&registration(&install_id, &app_id), "first")
         .await
         .unwrap();
      db.save_registration(&registration(&install_id, &app_id), "second")
         .await
         .unwrap();

      assert!(
         !db.roll_back_registration(&install_id, &app_id, "first")
            .await
            .unwrap()
      );
      assert!(
         db.get_registration(&install_id, &app_id)
            .await
            .unwrap()
            .is_some()
      );
      assert!(db.installation_hash(&install_id).await.unwrap().is_some());

      assert!(
         db.roll_back_registration(&install_id, &app_id, "second")
            .await
            .unwrap()
      );
      assert!(
         db.get_registration(&install_id, &app_id)
            .await
            .unwrap()
            .is_none()
      );
      assert!(db.installation_hash(&install_id).await.unwrap().is_none());
   }

   /// The reported leak: a concurrent request makes the second write report
   /// `Existing`, and the handler used to skip rollback on that, stranding a
   /// row with no listener. Rollback is keyed on the generation, not on that
   /// answer.
   #[tokio::test]
   async fn an_updating_write_still_rolls_itself_back() {
      let db = fresh_db().await;
      let secret = InstallSecret::from("secret");
      let install_id = InstallId::try_from("0123456789abcdef").unwrap();
      let app_id = AppId::trusted("com.app");

      db.claim_installation(&install_id, "hash", &secret)
         .await
         .unwrap();
      assert_eq!(
         db.save_registration(&registration(&install_id, &app_id), "first")
            .await
            .unwrap(),
         Claim::Enrolled
      );
      db.save_fcm_session(&install_id, &app_id, "session")
         .await
         .unwrap();
      assert_eq!(
         db.save_registration(&registration(&install_id, &app_id), "second")
            .await
            .unwrap(),
         Claim::Existing
      );

      assert!(
         db.roll_back_registration(&install_id, &app_id, "second")
            .await
            .unwrap()
      );
      assert_eq!(db.count_registrations().await.unwrap(), 0);
      assert!(db.installation_hash(&install_id).await.unwrap().is_none());
      // The FCM token outlives the rollback, so a later registration reuses it
      // instead of minting one the app's server never learns.
      assert_eq!(
         db.get_fcm_session(&install_id, &app_id).await.unwrap(),
         Some("session".to_owned())
      );
   }

   /// A rolled-back FCM registration and a live `UnifiedPush` registration can
   /// share one (install, app) pair, and the rollback owns only the FCM half.
   #[tokio::test]
   async fn rolling_back_spares_queued_unified_push_messages() {
      let db = fresh_db().await;
      let secret = InstallSecret::from("secret");
      let install_id = InstallId::try_from("0123456789abcdef").unwrap();
      let app_id = AppId::trusted("com.app");
      let connector_token = ConnectorToken::try_from("token").unwrap();

      db.claim_installation(&install_id, "hash", &secret)
         .await
         .unwrap();
      db.register_unified_push(&install_id, &app_id, &connector_token, "endpoint", None)
         .await
         .unwrap();
      db.save_registration(&registration(&install_id, &app_id), "gen")
         .await
         .unwrap();
      let queued = db
         .enqueue_unified_push_message(&install_id, &app_id, &connector_token, b"payload")
         .await
         .unwrap();
      db.enqueue_fcm_message(&install_id, &app_id, Some("p-1"), b"fcm")
         .await
         .unwrap()
         .unwrap();
      assert_eq!(db.outbox_len(&install_id).await.unwrap(), 2);

      db.roll_back_registration(&install_id, &app_id, "gen")
         .await
         .unwrap();

      let head = db
         .next_socket_message(&install_id, Cursor::default())
         .await
         .unwrap()
         .unwrap();
      assert_eq!(head.id, queued);
      assert_eq!(head.kind, MessageKind::UnifiedPush);
      assert_eq!(db.outbox_len(&install_id).await.unwrap(), 1);
      // The UnifiedPush registration is what keeps the install alive here.
      assert!(db.installation_hash(&install_id).await.unwrap().is_some());
   }

   /// A failed listener start rolls the install back, but only when it owns
   /// nothing, or a retry would strip a live tenant's identity.
   #[tokio::test]
   async fn rolling_back_an_install_spares_one_that_owns_something() {
      let db = fresh_db().await;
      let secret = InstallSecret::from("secret");
      let bare = InstallId::try_from("0123456789abcdef").unwrap();
      let with_registration = InstallId::try_from("fedcba9876543210").unwrap();
      let with_unified_push = InstallId::try_from("aaaabbbbccccdddd").unwrap();
      let app_id = AppId::trusted("com.app");

      for install in [&bare, &with_registration, &with_unified_push] {
         db.claim_installation(install, "hash", &secret)
            .await
            .unwrap();
      }
      db.save_registration(&registration(&with_registration, &app_id), "gen")
         .await
         .unwrap();
      db.register_unified_push(
         &with_unified_push,
         &app_id,
         &ConnectorToken::try_from("token").unwrap(),
         "endpoint",
         None,
      )
      .await
      .unwrap();

      assert!(db.delete_unused_installation(&bare).await.unwrap());
      assert!(
         !db.delete_unused_installation(&with_registration)
            .await
            .unwrap()
      );
      assert!(
         !db.delete_unused_installation(&with_unified_push)
            .await
            .unwrap()
      );
      assert!(db.installation_hash(&bare).await.unwrap().is_none());
      assert!(
         db.installation_hash(&with_registration)
            .await
            .unwrap()
            .is_some()
      );
      assert!(
         db.installation_hash(&with_unified_push)
            .await
            .unwrap()
            .is_some()
      );
   }

   /// An install with no registrations refreshes `updated_at` every time it
   /// re-authenticates, so only its age can retire it.
   #[tokio::test]
   async fn an_install_that_never_registers_is_reaped_on_age() {
      let db = fresh_db().await;
      let secret = InstallSecret::from("secret");
      let squatter = InstallId::try_from("0123456789abcdef").unwrap();
      let recent = InstallId::try_from("fedcba9876543210").unwrap();

      for install in [&squatter, &recent] {
         db.claim_installation(install, "hash", &secret)
            .await
            .unwrap();
      }
      db.backdate_installation_column(&squatter, "created_at", UNUSED_INSTALL_MAX_AGE_DAYS + 1)
         .await
         .unwrap();

      let outcome = db.prune_stale(90).await.unwrap();

      assert_eq!(
         outcome
            .installs
            .iter()
            .map(|i| i.install_id.clone())
            .collect::<Vec<_>>(),
         vec![squatter.clone()]
      );
      assert!(db.installation_hash(&squatter).await.unwrap().is_none());
      assert!(db.installation_hash(&recent).await.unwrap().is_some());
   }

   /// Refusing a refresh would brick a device the moment it hit the limit.
   #[tokio::test]
   async fn registration_quota_refuses_new_apps_but_not_refreshes() {
      let db = fresh_db().await;
      let install_id = InstallId::try_from("0123456789abcdef").unwrap();
      let other_install = InstallId::try_from("fedcba9876543210").unwrap();

      for index in 0..MAX_REGISTRATIONS_PER_INSTALL {
         let app_id = AppId::trusted(&format!("com.app{index}"));
         assert_eq!(
            db.save_registration(&registration(&install_id, &app_id), "gen")
               .await
               .unwrap(),
            Claim::Enrolled
         );
      }

      let overflow = AppId::trusted("com.overflow");
      assert_eq!(
         db.save_registration(&registration(&install_id, &overflow), "gen")
            .await
            .unwrap(),
         Claim::Refused(QuotaRejection::Install)
      );
      assert_eq!(
         db.save_registration(
            &registration(&install_id, &AppId::trusted("com.app0")),
            "gen"
         )
         .await
         .unwrap(),
         Claim::Existing
      );
      // The ceiling is per install, so a full tenant cannot deny anyone else.
      assert_eq!(
         db.save_registration(&registration(&other_install, &overflow), "gen")
            .await
            .unwrap(),
         Claim::Enrolled
      );
   }

   #[tokio::test]
   async fn unified_push_quota_is_per_install() {
      let db = fresh_db().await;
      let install_id = InstallId::try_from("0123456789abcdef").unwrap();
      let app_id = AppId::trusted("com.app");
      for index in 0..MAX_UNIFIED_PUSH_PER_INSTALL {
         let token = ConnectorToken::try_from(format!("token-{index}")).unwrap();
         assert!(matches!(
            db.register_unified_push(&install_id, &app_id, &token, &format!("ep-{index}"), None)
               .await
               .unwrap(),
            UnifiedPushClaim::Registered(_)
         ));
      }
      let overflow = ConnectorToken::try_from("token-overflow").unwrap();
      assert_eq!(
         db.register_unified_push(&install_id, &app_id, &overflow, "ep-overflow", None)
            .await
            .unwrap(),
         UnifiedPushClaim::Refused(QuotaRejection::Install)
      );
      // Retiring one frees the slot, so a full token rotation can recover.
      assert_eq!(
         db.delete_stale_unified_push_registrations(&install_id, slice::from_ref(&overflow))
            .await
            .unwrap(),
         MAX_UNIFIED_PUSH_PER_INSTALL
      );
      assert!(matches!(
         db.register_unified_push(&install_id, &app_id, &overflow, "ep-overflow", None)
            .await
            .unwrap(),
         UnifiedPushClaim::Registered(_)
      ));
   }

   /// The newest message is the one a user is waiting on.
   #[tokio::test]
   async fn outbox_ceiling_evicts_the_oldest_and_still_accepts() {
      let db = fresh_db().await;
      let install_id = InstallId::try_from("0123456789abcdef").unwrap();
      let other_install = InstallId::try_from("fedcba9876543210").unwrap();
      let app_id = AppId::trusted("com.app");

      db.seed_outbox(&install_id, &app_id, MAX_OUTBOX_ROWS_PER_INSTALL)
         .await
         .unwrap();
      db.seed_outbox(&other_install, &app_id, 4).await.unwrap();
      let oldest = db
         .next_socket_message(&install_id, Cursor::default())
         .await
         .unwrap()
         .unwrap();

      let accepted = db
         .enqueue_fcm_message(&install_id, &app_id, Some("newest"), b"newest")
         .await
         .unwrap()
         .unwrap();

      let head = db
         .next_socket_message(&install_id, Cursor::default())
         .await
         .unwrap()
         .unwrap();
      assert_ne!(
         head.id, oldest.id,
         "the oldest message survived the ceiling"
      );
      let queued = db.outbox_len(&install_id).await.unwrap();
      assert_eq!(queued, MAX_OUTBOX_ROWS_PER_INSTALL);
      // The arrival is deliverable and a bystander's queue is untouched.
      assert!(db.ack_socket_message(&install_id, accepted).await.unwrap());
      assert_eq!(db.outbox_len(&other_install).await.unwrap(), 4);
   }

   /// The live database's shape on 2026-08-21. Against a fresh schema the
   /// column drops would all be no-ops.
   const V4_SCHEMA: &str = "CREATE TABLE registrations (
            install_id TEXT NOT NULL,
            app_id TEXT NOT NULL,
            secret_hash TEXT NOT NULL,
            endpoint TEXT NOT NULL,
            fcm_token TEXT,
            firebase_app_id TEXT NOT NULL,
            firebase_project_id TEXT NOT NULL,
            firebase_api_key TEXT NOT NULL,
            cert_sha1 TEXT,
            app_version INTEGER,
            app_version_name TEXT,
            target_sdk INTEGER,
            transport TEXT NOT NULL DEFAULT 'unified_push',
            created_at TEXT DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
            PRIMARY KEY (install_id, app_id)
         );
         CREATE TABLE outbox (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            install_id TEXT NOT NULL,
            app_id TEXT NOT NULL,
            kind TEXT NOT NULL,
            transport TEXT NOT NULL,
            connector_token TEXT,
            persistent_id TEXT,
            payload BLOB NOT NULL,
            attempts INTEGER NOT NULL DEFAULT 0,
            next_attempt_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            created_at TEXT DEFAULT CURRENT_TIMESTAMP
         );
         CREATE UNIQUE INDEX outbox_fcm_persistent_id
            ON outbox(install_id, app_id, persistent_id)
            WHERE persistent_id IS NOT NULL;
         CREATE INDEX outbox_socket_order ON outbox(install_id, transport, id);
         CREATE INDEX outbox_up_due ON outbox(transport, next_attempt_at, id);";

   #[test]
   fn v4_database_drops_the_shim_columns_without_losing_rows() {
      let mut conn = rusqlite::Connection::open_in_memory().unwrap();
      conn.execute_batch(V4_SCHEMA).unwrap();
      conn
         .execute_batch(
            "INSERT INTO registrations
                (install_id, app_id, secret_hash, endpoint,
                 \
             firebase_app_id, firebase_project_id, firebase_api_key, transport)
                VALUES \
             ('0123456789abcdef', 'com.app', 'hash', 'https://n.example/t',
                        '1:123:android:abc', 'proj', 'key', 'websocket');
             INSERT INTO outbox (install_id, app_id, kind, transport, payload)
                VALUES ('0123456789abcdef', 'com.app', 'fcm', 'websocket', x'01');",
         )
         .unwrap();
      conn.pragma_update(None, "user_version", 4).unwrap();

      init_schema(&mut conn).unwrap();

      assert!(column_exists(&conn, "registrations", "generation").unwrap());
      for (table, column) in [
         ("registrations", "endpoint"),
         ("registrations", "transport"),
         ("outbox", "transport"),
      ] {
         assert!(
            !column_exists(&conn, table, column).unwrap(),
            "{table}.{column} survived the migration"
         );
      }
      // The socket cursor query orders on this index.
      let indexes = {
         let mut stmt = conn
            .prepare("SELECT name FROM sqlite_master WHERE type = 'index' AND tbl_name = 'outbox'")
            .unwrap();
         stmt
            .query_map([], |row| row.get::<_, String>(0))
            .unwrap()
            .collect::<rusqlite::Result<Vec<_>>>()
            .unwrap()
      };
      assert!(indexes.contains(&"outbox_socket_order".to_owned()));
      assert!(!indexes.contains(&"outbox_up_due".to_owned()));

      // The installation must be backfilled or the client cannot authenticate.
      let (app_id, key): (String, String) = conn
         .query_row(
            "SELECT app_id, firebase_api_key FROM registrations",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
         )
         .unwrap();
      assert_eq!((app_id.as_str(), key.as_str()), ("com.app", "key"));
      assert_eq!(
         conn
            .query_row("SELECT COUNT(*) FROM outbox", [], |row| {
               row.get::<_, i64>(0)
            })
            .unwrap(),
         1
      );
      assert_eq!(
         conn
            .query_row(
               "SELECT secret_hash FROM installations WHERE install_id = ?1",
               ["0123456789abcdef"],
               |row| row.get::<_, String>(0)
            )
            .unwrap(),
         "hash"
      );

      // A rollback and redeploy replays this.
      conn.pragma_update(None, "user_version", 4).unwrap();
      init_schema(&mut conn).unwrap();
      assert_eq!(
         conn
            .query_row("PRAGMA user_version", [], |row| row.get::<_, i32>(0))
            .unwrap(),
         SCHEMA_VERSION
      );
   }

   #[test]
   fn v3_database_gains_vapid_pubkey_without_losing_rows() {
      let mut conn = rusqlite::Connection::open_in_memory().unwrap();
      conn
         .execute_batch(
            "CREATE TABLE unified_push_registrations (
                install_id TEXT NOT NULL,
                app_id TEXT NOT NULL,
                connector_token TEXT NOT NULL,
                endpoint_token TEXT NOT NULL UNIQUE,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
                PRIMARY KEY (install_id, connector_token)
             );
             INSERT INTO unified_push_registrations
                (install_id, app_id, connector_token, endpoint_token)
                VALUES ('0123456789abcdef', 'com.app', 'connector-1', 'endpoint-1');",
         )
         .unwrap();
      conn.pragma_update(None, "user_version", 3).unwrap();

      init_schema(&mut conn).unwrap();

      assert!(column_exists(&conn, "unified_push_registrations", "vapid_pubkey").unwrap());

      let version = conn
         .query_row("PRAGMA user_version", [], |row| row.get::<_, i32>(0))
         .unwrap();
      assert_eq!(version, SCHEMA_VERSION);

      let (app_id, endpoint_token, vapid_pubkey): (String, String, Option<String>) = conn
         .query_row(
            "SELECT app_id, endpoint_token, vapid_pubkey
                  FROM unified_push_registrations WHERE install_id = ?1",
            ["0123456789abcdef"],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
         )
         .unwrap();
      assert_eq!(app_id, "com.app");
      assert_eq!(endpoint_token, "endpoint-1");
      assert_eq!(vapid_pubkey, None);

      conn.pragma_update(None, "user_version", 3).unwrap();
      init_schema(&mut conn).unwrap();
      let version = conn
         .query_row("PRAGMA user_version", [], |row| row.get::<_, i32>(0))
         .unwrap();
      assert_eq!(version, SCHEMA_VERSION);
      let row_count = conn
         .query_row(
            "SELECT COUNT(*) FROM unified_push_registrations",
            [],
            |row| row.get::<_, i64>(0),
         )
         .unwrap();
      assert_eq!(row_count, 1);
   }

   #[tokio::test]
   async fn prune_reaps_stale_installs_with_their_unified_push_state() {
      let db = fresh_db().await;
      let secret = InstallSecret::from("secret");
      let app_id = AppId::trusted("com.app");
      let stale_install = InstallId::try_from("0123456789abcdef").unwrap();
      let fresh_install = InstallId::try_from("fedcba9876543210").unwrap();
      let registered_install = InstallId::try_from("aaaabbbbccccdddd").unwrap();
      let stale_token = ConnectorToken::try_from("stale-token").unwrap();
      let fresh_token = ConnectorToken::try_from("fresh-token").unwrap();

      for install_id in [&stale_install, &fresh_install, &registered_install] {
         assert_eq!(
            db.claim_installation(install_id, "hash", &secret)
               .await
               .unwrap(),
            Claim::Enrolled
         );
      }
      db.register_unified_push(
         &stale_install,
         &app_id,
         &stale_token,
         "stale-endpoint",
         None,
      )
      .await
      .unwrap();
      db.register_unified_push(
         &fresh_install,
         &app_id,
         &fresh_token,
         "fresh-endpoint",
         None,
      )
      .await
      .unwrap();
      db.enqueue_unified_push_message(&stale_install, &app_id, &stale_token, b"queued")
         .await
         .unwrap();
      db.save_registration(&registration(&registered_install, &app_id), "gen")
         .await
         .unwrap();
      db.backdate_installation(&stale_install, 91).await.unwrap();
      db.backdate_installation(&registered_install, 91)
         .await
         .unwrap();

      let outcome = db.prune_stale(90).await.unwrap();

      assert!(outcome.pairs.is_empty());
      assert_eq!(outcome.installs.len(), 1);
      assert_eq!(outcome.installs[0].install_id, stale_install);
      assert_eq!(outcome.installs[0].unified_push_registrations, 1);
      assert_eq!(outcome.installs[0].outbox_messages, 1);
      assert!(
         db.get_unified_push_endpoint("stale-endpoint")
            .await
            .unwrap()
            .is_none()
      );
      assert!(
         db.installation_secret(&stale_install)
            .await
            .unwrap()
            .is_none()
      );
      assert!(
         db.get_unified_push_endpoint("fresh-endpoint")
            .await
            .unwrap()
            .is_some()
      );
      assert!(
         db.installation_secret(&fresh_install)
            .await
            .unwrap()
            .is_some()
      );
      // A fresh registration keeps its installation alive even when the
      // installation row itself aged out.
      assert!(
         db.installation_secret(&registered_install)
            .await
            .unwrap()
            .is_some()
      );
   }

   #[tokio::test]
   async fn socket_outbox_replays_until_ack_and_deduplicates_fcm() {
      let db = fresh_db().await;
      let install_id = InstallId::try_from("0123456789abcdef").unwrap();
      let app_id = AppId::trusted("com.app");
      let id = db
         .enqueue_fcm_message(
            &install_id,
            &app_id,
            Some("persistent-1"),
            br#"{"google.message_id":"message-1"}"#,
         )
         .await
         .unwrap()
         .unwrap();

      assert_eq!(
         db.enqueue_fcm_message(&install_id, &app_id, Some("persistent-1"), b"duplicate",)
            .await
            .unwrap(),
         None
      );
      let pending = db
         .next_socket_message(&install_id, Cursor::default())
         .await
         .unwrap()
         .unwrap();
      assert_eq!(pending.id, id);
      assert_eq!(pending.kind, MessageKind::Fcm);
      assert!(db.ack_socket_message(&install_id, id).await.unwrap());
      assert!(
         db.next_socket_message(&install_id, Cursor::default())
            .await
            .unwrap()
            .is_none()
      );
   }
}
