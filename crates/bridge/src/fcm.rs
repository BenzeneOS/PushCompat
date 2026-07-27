//! FCM listener management.
//!
//! Holds one MCS connection per registered app and hands every message to the
//! delivery manager.

use std::{
   collections::HashMap,
   sync::{
      Arc,
      Mutex,
      atomic::{
         AtomicU64,
         Ordering,
      },
   },
   time::Duration,
};

use anyhow::Result;
use futures_util::StreamExt as _;
use pushcompat_listener::{
   AppRegistration,
   AppRegistrationState,
   DeviceSession,
   DeviceSessionState,
   FcmCredentials,
   MCS_HEARTBEAT_INTERVAL,
   MCS_IDLE_TIMEOUT,
   Message,
   MessageTag,
   decode_login_response,
   new_heartbeat_ping,
   write_all_with_deadline,
};
use serde::{
   Deserialize,
   Serialize,
};
use tokio::{
   sync::{
      Mutex as AsyncMutex,
      Semaphore,
      mpsc,
   },
   task::JoinHandle,
   time::{
      Instant,
      MissedTickBehavior,
      interval_at,
      sleep,
   },
};
use tracing::{
   error,
   info,
   warn,
};

use crate::{
   db::Database,
   delivery::DeliveryManager,
   types::{
      AppId,
      InstallId,
   },
};

/// MCS carries the ack list inline in the login packet, so it cannot grow
/// unbounded.
const MAX_ACK_HISTORY: usize = 500;

type ListenerKey = (InstallId, AppId);

/// Registering talks to Google and can take minutes, so the map is only locked
/// to look a handle up or swap one in, never across an `await`. Concurrent
/// starts for one app serialize on a per-key gate instead, so a retry cannot
/// open two MCS sessions for the same registration.
pub struct FcmManager {
   listeners:      Mutex<HashMap<ListenerKey, ListenerHandle>>,
   starts:         Mutex<HashMap<ListenerKey, Arc<AsyncMutex<()>>>>,
   /// Bounds how many registrations are talking to Google at once.
   start_permits:  Semaphore,
   /// HTTP client for FCM registration.
   http_client:    reqwest::Client,
   dead_listeners: AtomicU64,
}

const MAX_CONCURRENT_STARTS: usize = 16;

#[derive(PartialEq, Eq)]
struct ListenerConfig {
   firebase_app_id:     String,
   firebase_project_id: String,
   firebase_api_key:    String,
   cert_sha1:           Option<String>,
   app_version:         Option<i32>,
   app_version_name:    Option<String>,
   target_sdk:          Option<i32>,
}

struct ListenerHandle {
   /// Channel to stop the listener.
   stop_tx:   mpsc::Sender<()>,
   /// FCM token for this registration.
   fcm_token: String,
   config:    ListenerConfig,
   task:      Option<JoinHandle<()>>,
}

impl ListenerHandle {
   fn is_dead(&self) -> bool {
      self.task.as_ref().is_none_or(JoinHandle::is_finished)
   }
}

#[derive(Serialize, Deserialize)]
struct StoredFcmRegistration {
   gcm_session: DeviceSessionState,
   gcm_token:   StoredGcmToken,
   credentials: FcmCredentials,
}

#[derive(Serialize, Deserialize)]
struct StoredGcmToken {
   token: String,
}

impl StoredFcmRegistration {
   fn snapshot(device: &DeviceSession, app: &AppRegistration) -> Self {
      let app = app.state();
      Self {
         gcm_session: device.state(),
         gcm_token:   StoredGcmToken {
            token: app.fcm_token,
         },
         credentials: app.credentials,
      }
   }

   fn restore(self) -> (DeviceSession, AppRegistration) {
      (
         DeviceSession::restore(self.gcm_session),
         AppRegistration::restore(AppRegistrationState {
            fcm_token:   self.gcm_token.token,
            credentials: self.credentials,
         }),
      )
   }
}

impl FcmManager {
   pub fn new() -> Self {
      pushcompat_listener::install_crypto_provider();
      Self {
         listeners:      Mutex::new(HashMap::new()),
         starts:         Mutex::new(HashMap::new()),
         start_permits:  Semaphore::new(MAX_CONCURRENT_STARTS),
         http_client:    pushcompat_listener::http_client_builder()
            .http1_only()
            .timeout(Duration::from_secs(15))
            .build()
            .expect("failed to build HTTP client"),
         dead_listeners: AtomicU64::new(0),
      }
   }

   pub fn dead_listener_count(&self) -> u64 {
      self.dead_listeners.load(Ordering::Relaxed)
   }

   pub fn active_count(&self) -> usize {
      self
         .listeners
         .lock()
         .expect("fcm listener lock poisoned")
         .values()
         .filter(|handle| !handle.is_dead())
         .count()
   }

   fn running_token(&self, key: &ListenerKey, config: &ListenerConfig) -> Option<String> {
      let listeners = self.listeners.lock().expect("fcm listener lock poisoned");
      let handle = listeners.get(key);
      let dead = handle.is_some_and(ListenerHandle::is_dead);
      let token = handle
         .filter(|handle| !handle.is_dead() && &handle.config == config)
         .map(|handle| handle.fcm_token.clone());
      drop(listeners);

      if dead {
         self.dead_listeners.fetch_add(1, Ordering::Relaxed);
         error!(
            "FCM listener task for {}/{} died unexpectedly, rebuilding",
            key.0, key.1
         );
      }
      token
   }

   fn take(&self, key: &ListenerKey) {
      let handle = self
         .listeners
         .lock()
         .expect("fcm listener lock poisoned")
         .remove(key);
      if let Some(handle) = handle {
         let _ = handle.stop_tx.try_send(());
         if let Some(task) = handle.task {
            task.abort();
         }
      }
   }

   fn start_gate(&self, key: &ListenerKey) -> Arc<AsyncMutex<()>> {
      Arc::clone(
         self
            .starts
            .lock()
            .expect("fcm start lock poisoned")
            .entry(key.clone())
            .or_default(),
      )
   }

   pub async fn start_listener(
      &self,
      registration: ListenerRegistration,
      db: Arc<Database>,
      delivery: Arc<DeliveryManager>,
   ) -> Result<String> {
      let ListenerRegistration {
         install_id,
         app_id,
         firebase_app_id,
         firebase_project_id,
         firebase_api_key,
         cert_sha1,
         app_version,
         app_version_name,
         target_sdk,
      } = registration;
      let key = (install_id.clone(), app_id.clone());
      let label = format!("{install_id}/{app_id}");
      let config = ListenerConfig {
         firebase_app_id: firebase_app_id.clone(),
         firebase_project_id: firebase_project_id.clone(),
         firebase_api_key: firebase_api_key.clone(),
         cert_sha1: cert_sha1.clone(),
         app_version,
         app_version_name: app_version_name.clone(),
         target_sdk,
      };
      let gate = self.start_gate(&key);
      let _starting = gate.lock().await;
      // A concurrent start for this app may have finished while we queued.
      if let Some(token) = self.running_token(&key, &config) {
         info!("Keeping unchanged FCM listener for {}", label);
         // A relogin inside the listener task can rotate the token after the
         // handle cached it, so the stored session wins.
         let token = stored_session_token(&db, &install_id, &app_id)
            .await
            .unwrap_or(token);
         return Ok(token);
      }

      self.take(&key);

      let _starting_permit = self
         .start_permits
         .acquire()
         .await
         .map_err(|_| anyhow::anyhow!("listener start permits closed"))?;

      // Extract sender_id from firebase_app_id
      // Format: "1:<sender_id>:android:<hash>"
      let sender_id = extract_sender_id(&firebase_app_id)?;

      // Build FCM credentials
      let credentials = FcmCredentials {
         sender_id: sender_id.clone(),
         api_key: firebase_api_key,
         app_id: firebase_app_id,
         project_id: firebase_project_id,
         package_name: app_id.as_ref().to_owned(),
         cert_sha1,
         app_version,
         app_version_name,
         target_sdk,
      };

      // A load error must not read as absence. Re-registering would mint a
      // token the app's server never learns, so fail the start and let a
      // later registration retry.
      let (device, app_registration) = match db.get_fcm_session(&install_id, &app_id).await {
         Err(error) => {
            error!("Failed to load saved FCM session for {label}: {error}");
            return Err(error);
         },
         Ok(Some(session_json)) => {
            match serde_json::from_str::<StoredFcmRegistration>(&session_json) {
               Ok(existing) => {
                  let restored = existing.restore();
                  info!(
                     "Reusing existing FCM session for {} (token: {}...)",
                     label,
                     &restored.1.fcm_token()[..20.min(restored.1.fcm_token().len())]
                  );
                  restored
               },
               Err(error) => {
                  warn!(
                     "Failed to deserialize saved session for {}: {}, re-registering",
                     label, error
                  );
                  register_app(&self.http_client, credentials.clone()).await?
               },
            }
         },
         Ok(None) => {
            info!(
               "Registering with FCM for app: {} (sender_id: {}, cert: {})",
               label,
               sender_id,
               credentials.cert_sha1.as_deref().unwrap_or("none")
            );
            register_app(&self.http_client, credentials.clone()).await?
         },
      };

      let fcm_token = app_registration.fcm_token().to_owned();
      info!(
         "Got FCM token for {}: {}...",
         label,
         &fcm_token[..20.min(fcm_token.len())]
      );

      // Save registration for reconnection
      save_registration_snapshot(&db, &install_id, &app_id, &device, &app_registration).await;

      // Create stop channel
      let (stop_tx, stop_rx) = mpsc::channel(1);

      // Clone values for the listener task
      let fcm_token_clone = fcm_token.clone();

      // Spawn listener task
      let db_for_listener = Arc::clone(&db);
      let http_for_listener = self.http_client.clone();
      let credentials_for_listener = credentials.clone();
      let task = tokio::spawn(async move {
         run_listener(
            ListenerRuntime {
               install_id: install_id.clone(),
               app_id: app_id.clone(),
               sender_id,
               device,
               credentials: credentials_for_listener,
               http: http_for_listener,
               db: db_for_listener,
               delivery,
            },
            stop_rx,
         )
         .await;
      });

      self
         .listeners
         .lock()
         .expect("fcm listener lock poisoned")
         .insert(key, ListenerHandle {
            stop_tx,
            fcm_token: fcm_token_clone,
            config,
            task: Some(task),
         });

      Ok(fcm_token)
   }

   pub fn stop_listener(&self, install_id: &InstallId, app_id: &AppId) {
      let key = (install_id.clone(), app_id.clone());
      self.take(&key);
      self
         .starts
         .lock()
         .expect("fcm start lock poisoned")
         .remove(&key);
      info!("Stopped FCM listener for {}/{}", install_id, app_id);
   }
}

pub struct ListenerRegistration {
   pub install_id:          InstallId,
   pub app_id:              AppId,
   pub firebase_app_id:     String,
   pub firebase_project_id: String,
   pub firebase_api_key:    String,
   pub cert_sha1:           Option<String>,
   pub app_version:         Option<i32>,
   pub app_version_name:    Option<String>,
   pub target_sdk:          Option<i32>,
}

async fn register_app(
   http: &reqwest::Client,
   credentials: FcmCredentials,
) -> Result<(DeviceSession, AppRegistration), pushcompat_listener::Error> {
   let device = DeviceSession::fresh(http).await?;
   let app = AppRegistration::register(http, &device, credentials).await?;
   Ok((device, app))
}

async fn save_registration_snapshot(
   db: &Database,
   install_id: &InstallId,
   app_id: &AppId,
   device: &DeviceSession,
   registration: &AppRegistration,
) {
   let stored = StoredFcmRegistration::snapshot(device, registration);
   match serde_json::to_string(&stored) {
      Ok(json) => {
         if let Err(error) = db.save_fcm_session(install_id, app_id, &json).await {
            error!("Failed to save FCM session for {install_id}/{app_id}: {error}");
         }
      },
      Err(error) => {
         error!("Failed to serialize FCM session for {install_id}/{app_id}: {error}");
      },
   }
}

async fn stored_session_token(
   db: &Database,
   install_id: &InstallId,
   app_id: &AppId,
) -> Option<String> {
   let session_json = db
      .get_fcm_session(install_id, app_id)
      .await
      .ok()
      .flatten()?;
   let stored = serde_json::from_str::<StoredFcmRegistration>(&session_json).ok()?;
   Some(stored.gcm_token.token)
}

pub fn validate_credentials(
   firebase_app_id: &str,
   firebase_project_id: &str,
   firebase_api_key: &str,
) -> Result<()> {
   if firebase_project_id.trim().is_empty() || firebase_api_key.trim().is_empty() {
      anyhow::bail!("firebase_project_id and firebase_api_key must not be empty");
   }
   extract_sender_id(firebase_app_id).map(|_| ())
}

/// Extract `sender_id` from Firebase app ID.
/// Format: "1:<`sender_id>:android`:<hash>" or "1:<`sender_id>:web`:<hash>".
fn extract_sender_id(firebase_app_id: &str) -> Result<String> {
   let parts = firebase_app_id.split(':').collect::<Vec<&str>>();
   if parts.len() >= 4
      && parts[0] == "1"
      && !parts[1].is_empty()
      && parts[1].bytes().all(|byte| byte.is_ascii_digit())
      && matches!(parts[2], "android" | "web")
      && !parts[3].is_empty()
   {
      Ok(parts[1].to_owned())
   } else {
      anyhow::bail!("Invalid firebase_app_id format: {firebase_app_id}")
   }
}

struct ListenerRuntime {
   install_id:  InstallId,
   app_id:      AppId,
   sender_id:   String,
   device:      DeviceSession,
   credentials: FcmCredentials,
   http:        reqwest::Client,
   db:          Arc<Database>,
   delivery:    Arc<DeliveryManager>,
}

async fn run_listener(runtime: ListenerRuntime, mut stop_rx: mpsc::Receiver<()>) {
   let ListenerRuntime {
      install_id,
      app_id,
      sender_id,
      mut device,
      credentials,
      http,
      db,
      delivery,
   } = runtime;
   let label = format!("{install_id}/{app_id}");
   info!("Starting FCM listener for {label}");
   let mut relogin_attempts = 0_u32;

   // Seeded from disk
   let mut persistent_ids = db
      .recent_acks(&install_id, &app_id, MAX_ACK_HISTORY)
      .await
      .unwrap_or_default();

   // The query is newest-first, while this in-memory queue evicts from the
   // front as newer ids arrive.
   persistent_ids.reverse();
   info!(
      "Restored {} acked message ids for {label}",
      persistent_ids.len(),
   );

   loop {
      // Check if we should stop
      if stop_rx.try_recv().is_ok() {
         info!("FCM listener stopped for {label}");
         break;
      }

      // Connect to mtalk.google.com
      info!(
         "Sending {} persistent ids in MCS login for {label}",
         persistent_ids.len(),
      );
      let mut stream = match device.connect(persistent_ids.clone()).await {
         Ok(stream) => stream,
         Err(error) => {
            error!("FCM connection failed for {label}: {error}");
            sleep(Duration::from_secs(30)).await;
            continue;
         },
      };

      info!("FCM connection established for {label}");
      let mut last_acked_stream_id = None;
      let mut login_rejected = false;
      let mut heartbeat = interval_at(
         Instant::now() + MCS_HEARTBEAT_INTERVAL,
         MCS_HEARTBEAT_INTERVAL,
      );
      heartbeat.set_missed_tick_behavior(MissedTickBehavior::Delay);
      let idle_deadline = sleep(MCS_IDLE_TIMEOUT);
      tokio::pin!(idle_deadline);

      // Listen for messages
      loop {
         tokio::select! {
             _ = stop_rx.recv() => {
                 info!("FCM listener stopped for {}", label);
                 return;
             }

             () = &mut idle_deadline => {
                 warn!("FCM connection for {label} exceeded the MCS idle deadline");
                 break;
             }

             _ = heartbeat.tick() => {
                 let ping = new_heartbeat_ping(stream.last_stream_id_received());
                 if ping.is_empty() {
                     error!("Failed to serialize MCS heartbeat ping for {label}");
                     break;
                 }
                 if let Err(error) = write_all_with_deadline(&mut *stream, &ping).await {
                     error!("Failed to send MCS heartbeat ping for {label}: {error}");
                     break;
                 }
             }

             msg = stream.next() => {
                 idle_deadline
                     .as_mut()
                     .reset(Instant::now() + MCS_IDLE_TIMEOUT);
                 let stream_id = stream.last_stream_id_received();
                 match msg {
                     Some(Ok(Message::Data(data))) => {
                         let payload_len = data.raw_data.as_ref().map_or(0, Vec::len);
                         info!(
                             "Received FCM message for {}: {} bytes, persistent_id: {:?}, from: {:?}",
                             label,
                             payload_len,
                             data.persistent_id,
                             data.from
                         );
                         let is_redelivery = data
                             .persistent_id
                             .as_ref()
                             .is_some_and(|pid| persistent_ids.contains(pid));
                         if is_redelivery {
                             info!(
                                 "Received redelivered FCM message for {label}: persistent_id={:?}",
                                 data.persistent_id,
                             );
                         }

                         // Rebuild what an Android FCM intent carries: the app payload
                         // plus the stanza metadata. `google.message_id` in particular is
                         // mandatory — the Firebase SDK drops messages that lack it — and it
                         // lives on the stanza, not in app_data.
                         let body = build_intent_payload(&data, &sender_id);
                         if body.is_empty() {
                             warn!("Empty payload in FCM message for {}", label);
                         }

                         if let Err(error) = delivery
                             .enqueue_fcm(
                                 &install_id,
                                 &app_id,
                                 data.persistent_id.as_deref(),
                                 &body,
                             )
                             .await
                         {
                             error!("Failed to persist FCM message for {label}: {error}");
                             break;
                         }

                         if let Some(pid) = &data.persistent_id
                             && !is_redelivery {
                                 persistent_ids.push(pid.clone());
                                 if persistent_ids.len() > MAX_ACK_HISTORY {
                                     persistent_ids.remove(0);
                                 }
                             }

                         let ack = pushcompat_listener::new_stream_ack(stream_id);
                         if let Err(error) = write_all_with_deadline(&mut *stream, &ack).await {
                             error!("Failed to acknowledge FCM message for {label}: {error}");
                             break;
                         }
                         last_acked_stream_id = Some(stream_id);

                     }

                     Some(Ok(Message::HeartbeatPing)) => {
                         let ack = pushcompat_listener::new_heartbeat_ack(stream_id);
                         if let Err(error) = write_all_with_deadline(&mut *stream, &ack).await {
                             error!("Failed to send heartbeat ack for {label}: {error}");
                             break; // Reconnect
                         }
                         last_acked_stream_id = Some(stream_id);
                     }

                     Some(Ok(Message::HeartbeatAck)) => {}

                     Some(Ok(Message::Other(tag, body))) => {
                         if tag == MessageTag::LoginResponse as u8 {
                             match decode_login_response(&body) {
                                 Ok(response) => {
                                     if response.error_code.is_some() {
                                         error!(
                                             "MCS login rejected for {label}: error_code={:?}, error_message={:?}, error_type={:?}",
                                             response.error_code,
                                             response.error_message,
                                             response.error_type,
                                         );
                                         login_rejected = true;
                                         break;
                                     }
                                     relogin_attempts = 0;
                                     info!(
                                         "MCS login response for {label}: id={}, stream_id={:?}, last_stream_id_received={:?}, server_timestamp={:?}",
                                         response.id,
                                         response.stream_id,
                                         response.last_stream_id_received,
                                         response.server_timestamp,
                                     );
                                 },
                                 Err(error) => warn!(
                                     "Failed to decode MCS login response for {label}: {error}",
                                 ),
                             }
                         } else {
                             warn!("Unknown FCM message type {} for {}", tag, label);
                         }
                     }

                     Some(Err(error)) => {
                         error!("FCM receive error for {label}: {error}");
                         break; // Reconnect
                     }

                     None => {
                         warn!("FCM stream ended for {}", label);
                         break; // Reconnect
                     }
                 }
             }
         }
      }

      if login_rejected {
         relogin_attempts += 1;
         let delay_seconds = 30_u64 << relogin_attempts.saturating_sub(1).min(6);
         warn!(
            "Recovering the MCS session for {label} in {delay_seconds}s (attempt \
             {relogin_attempts})"
         );
         sleep(Duration::from_secs(delay_seconds)).await;
         match reestablish_registration(&http, &mut device, &credentials).await {
            Ok(registration) => {
               info!(
                  "Re-registered {label} with FCM after login rejection (token: {}...)",
                  &registration.fcm_token()[..20.min(registration.fcm_token().len())]
               );
               save_registration_snapshot(&db, &install_id, &app_id, &device, &registration).await;
            },
            Err(error) => {
               error!("Failed to re-establish FCM registration for {label}: {error}");
            },
         }
         continue;
      }

      // Wait before reconnecting
      warn!(
         "FCM connection lost for {label}, last acknowledged stream id: {:?}, reconnecting in \
          5s...",
         last_acked_stream_id,
      );
      sleep(Duration::from_secs(5)).await;
   }
}

async fn reestablish_registration(
   http: &reqwest::Client,
   device: &mut DeviceSession,
   credentials: &FcmCredentials,
) -> Result<AppRegistration, pushcompat_listener::Error> {
   device.refresh(http).await?;
   AppRegistration::register(http, device, credentials.clone()).await
}

fn build_intent_payload(data: &pushcompat_listener::DataMessage, sender_id: &str) -> Vec<u8> {
   let mut fields = serde_json::Map::new();
   for (key, value) in &data.app_data {
      fields.insert(key.clone(), serde_json::Value::String(value.clone()));
   }
   if let Some(raw) = &data.raw_data {
      match serde_json::from_slice::<serde_json::Value>(raw) {
         Ok(serde_json::Value::Object(raw_fields)) => {
            for (key, raw_value) in raw_fields {
               let value = match raw_value {
                  serde_json::Value::String(value) => value,
                  other_value => other_value.to_string(),
               };
               fields.insert(key, serde_json::Value::String(value));
            }
         },
         _ => {
            fields.insert(
               "message".into(),
               serde_json::Value::String(String::from_utf8_lossy(raw).into_owned()),
            );
         },
      }
   }

   let message_id = data.id.clone().or_else(|| data.persistent_id.clone());
   for (key, value) in [
      ("google.message_id", message_id),
      (
         "from",
         Some(data.from.clone().unwrap_or_else(|| sender_id.to_owned())),
      ),
      ("google.c.sender.id", Some(sender_id.to_owned())),
      ("collapse_key", data.collapse_key.clone()),
   ] {
      if let Some(value) = value {
         fields.insert(key.into(), serde_json::Value::String(value));
      }
   }
   serde_json::to_vec(&fields).unwrap_or_default()
}

impl Default for FcmManager {
   fn default() -> Self {
      Self::new()
   }
}
