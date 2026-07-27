//! Per-install registration authentication.

use axum::http::{
   HeaderMap,
   StatusCode,
   header::AUTHORIZATION,
};
use data_encoding::HEXLOWER;
use sha2::{
   Digest as _,
   Sha256,
};
use subtle::ConstantTimeEq as _;

use crate::{
   db::{
      Claim,
      Database,
      QuotaRejection,
   },
   limit::RateLimiter,
   types::{
      AppId,
      InstallId,
      InstallSecret,
   },
};

pub fn hash_secret(secret: &InstallSecret) -> String {
   HEXLOWER.encode(&Sha256::digest(secret.expose().as_bytes()))
}

pub fn verify_secret(secret: &InstallSecret, stored_hash: &str) -> bool {
   let Ok(stored) = HEXLOWER.decode(stored_hash.as_bytes()) else {
      return false;
   };
   bool::from(
      Sha256::digest(secret.expose().as_bytes())
         .as_slice()
         .ct_eq(&stored),
   )
}

const MIN_SECRET_LEN: usize = 32;
const MAX_SECRET_LEN: usize = 128;

pub fn bearer_secret(headers: &HeaderMap) -> Option<InstallSecret> {
   let value = headers.get(AUTHORIZATION)?.to_str().ok()?;
   let (scheme, secret) = value.split_once(' ')?;
   if !scheme.eq_ignore_ascii_case("bearer") {
      return None;
   }
   let secret = secret.trim();
   if secret.len() < MIN_SECRET_LEN || secret.len() > MAX_SECRET_LEN {
      return None;
   }
   Some(InstallSecret::from(secret))
}

#[derive(Debug)]
pub struct RegisterIdentity {
   pub install_id:  InstallId,
   pub secret_hash: String,
}

/// Writes nothing, so a request rejected later leaves no row an attacker can
/// keep alive. Call [`Pending::claim`] once the whole request is known good.
pub async fn authenticate_register(
   db: &Database,
   install_id: Option<&str>,
   secret: Option<&InstallSecret>,
   limits: &RegisterLimits<'_>,
) -> Result<Pending, (StatusCode, String)> {
   let (Some(raw_install_id), Some(secret)) = (install_id, secret) else {
      return Err((
         StatusCode::UNAUTHORIZED,
         "install_id and secret required".to_owned(),
      ));
   };
   let install_id = InstallId::try_from(raw_install_id)
      .map_err(|_| (StatusCode::BAD_REQUEST, "invalid install_id".to_owned()))?;

   let stored_hash = db
      .installation_hash(&install_id)
      .await
      .map_err(|error| internal(&error))?;
   if stored_hash.is_some() {
      if !limits.per_install.acquire(install_id.as_ref()) {
         return Err((
            StatusCode::TOO_MANY_REQUESTS,
            "too many registration requests".to_owned(),
         ));
      }
   } else if !limits.enrollment.acquire(ENROLLMENT_BUCKET) {
      return Err((
         StatusCode::TOO_MANY_REQUESTS,
         "enrollment is rate limited".to_owned(),
      ));
   }

   let secret_hash = hash_secret(secret);
   if stored_hash
      .as_deref()
      .is_some_and(|stored| !verify_secret(secret, stored))
   {
      return Err((StatusCode::UNAUTHORIZED, "invalid credentials".to_owned()));
   }

   Ok(Pending {
      install_id,
      secret_hash,
      secret: InstallSecret::from(secret.expose()),
   })
}

/// An authenticated identity that has not been written yet. Deliberately not
/// Debug, it carries the bearer secret.
pub struct Pending {
   install_id:  InstallId,
   secret_hash: String,
   secret:      InstallSecret,
}

impl Pending {
   pub const fn install_id(&self) -> &InstallId {
      &self.install_id
   }

   pub async fn claim(self, db: &Database) -> Result<RegisterIdentity, (StatusCode, String)> {
      match db
         .claim_installation(&self.install_id, &self.secret_hash, &self.secret)
         .await
         .map_err(|error| internal(&error))?
      {
         Claim::Enrolled | Claim::Existing => {},
         Claim::Denied => {
            return Err((StatusCode::UNAUTHORIZED, "invalid credentials".to_owned()));
         },
         Claim::Refused(rejection) => return Err(quota(rejection)),
      }
      Ok(RegisterIdentity {
         install_id:  self.install_id,
         secret_hash: self.secret_hash,
      })
   }
}

/// For callers with nothing left to validate once authenticated. `/register`
/// uses the two-phase form instead, because its Firebase check can still
/// reject.
pub async fn authorize_register(
   db: &Database,
   _app_id: &AppId,
   install_id: Option<&str>,
   secret: Option<&InstallSecret>,
   limits: &RegisterLimits<'_>,
) -> Result<RegisterIdentity, (StatusCode, String)> {
   authenticate_register(db, install_id, secret, limits)
      .await?
      .claim(db)
      .await
}

/// Server-wide budget, so every new identity shares one bucket.
const ENROLLMENT_BUCKET: &str = "enrollment";

pub struct RegisterLimits<'a> {
   pub enrollment:  &'a RateLimiter,
   pub per_install: &'a RateLimiter,
}

pub fn quota(rejection: QuotaRejection) -> (StatusCode, String) {
   match rejection {
      QuotaRejection::Install => {
         (
            StatusCode::TOO_MANY_REQUESTS,
            "install has reached its registration limit".to_owned(),
         )
      },
      QuotaRejection::Server => {
         (
            StatusCode::SERVICE_UNAVAILABLE,
            "server is at capacity".to_owned(),
         )
      },
   }
}

pub async fn authorize_unregister(
   db: &Database,
   _app_id: &AppId,
   install_id: Option<&str>,
   secret: Option<&InstallSecret>,
) -> Result<InstallId, (StatusCode, String)> {
   let (Some(raw_install_id), Some(secret)) = (install_id, secret) else {
      return Err((
         StatusCode::UNAUTHORIZED,
         "install_id and secret required".to_owned(),
      ));
   };
   let Ok(install_id) = InstallId::try_from(raw_install_id) else {
      return Err((StatusCode::UNAUTHORIZED, "invalid credentials".to_owned()));
   };
   if db
      .verify_installation(install_id.as_ref(), secret)
      .await
      .map_err(|error| internal(&error))?
   {
      Ok(install_id)
   } else {
      Err((StatusCode::UNAUTHORIZED, "invalid credentials".to_owned()))
   }
}

fn internal(error: &anyhow::Error) -> (StatusCode, String) {
   tracing::error!("database error: {error}");
   (
      StatusCode::INTERNAL_SERVER_ERROR,
      "database error".to_owned(),
   )
}

#[cfg(test)]
mod tests {
   use std::time::Duration;

   use axum::http::StatusCode;

   use super::*;
   use crate::{
      db::{
         Database,
         Registration,
      },
      fcm::validate_credentials,
   };

   const INSTALL_A: &str = "0123456789abcdef0123456789abcdef";
   const INSTALL_B: &str = "fedcba9876543210fedcba9876543210";

   async fn test_db() -> Database {
      Database::new(":memory:").await.unwrap()
   }

   fn reg(install_id: &str, app_id: &str, secret_hash: &str) -> Registration {
      Registration {
         install_id:          InstallId::try_from(install_id).unwrap(),
         app_id:              AppId::trusted(app_id),
         secret_hash:         secret_hash.to_owned(),
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

   struct Limiters {
      enrollment:  RateLimiter,
      per_install: RateLimiter,
   }

   impl Limiters {
      fn generous() -> Self {
         Self {
            enrollment:  RateLimiter::new(1000, Duration::from_secs(1)),
            per_install: RateLimiter::new(1000, Duration::from_secs(1)),
         }
      }

      fn limits(&self) -> RegisterLimits<'_> {
         RegisterLimits {
            enrollment:  &self.enrollment,
            per_install: &self.per_install,
         }
      }
   }

   #[tokio::test]
   async fn first_registration_claims_then_secret_is_enforced() {
      let db = test_db().await;
      let app_id = AppId::trusted("com.app");
      let secret_a = InstallSecret::from("s1");
      let limiters = Limiters::generous();

      let identity = authorize_register(
         &db,
         &app_id,
         Some(INSTALL_A),
         Some(&secret_a),
         &limiters.limits(),
      )
      .await
      .unwrap();
      assert_eq!(identity.install_id.as_ref(), INSTALL_A);
      let mut registration = reg(INSTALL_A, "com.app", "");
      registration.secret_hash = identity.secret_hash;
      db.save_registration(&registration, "gen").await.unwrap();

      // Same secret: allowed
      authorize_register(
         &db,
         &app_id,
         Some(INSTALL_A),
         Some(&secret_a),
         &limiters.limits(),
      )
      .await
      .unwrap();
      // Wrong secret: 401
      let secret_b = InstallSecret::from("s2");
      let err = authorize_register(
         &db,
         &app_id,
         Some(INSTALL_A),
         Some(&secret_b),
         &limiters.limits(),
      )
      .await
      .unwrap_err();
      assert_eq!(err.0, StatusCode::UNAUTHORIZED);

      // Malformed install_id is a 400, distinct from a credential failure.
      let err = authorize_register(
         &db,
         &app_id,
         Some("i1"),
         Some(&secret_a),
         &limiters.limits(),
      )
      .await
      .unwrap_err();
      assert_eq!(err.0, StatusCode::BAD_REQUEST);
   }

   /// The reproduced attack was a rejected first registration leaving a row the
   /// attacker could then keep alive, so authentication must write nothing.
   #[tokio::test]
   async fn authentication_alone_persists_nothing() {
      let db = test_db().await;
      let secret = InstallSecret::from("s1");
      let limiters = Limiters::generous();
      let install = InstallId::try_from(INSTALL_A).unwrap();

      let pending = authenticate_register(&db, Some(INSTALL_A), Some(&secret), &limiters.limits())
         .await
         .unwrap();
      assert!(db.installation_hash(&install).await.unwrap().is_none());

      pending.claim(&db).await.unwrap();
      assert!(db.installation_hash(&install).await.unwrap().is_some());
   }

   /// Empty-but-present credentials reached `claim()` and left a durable row
   /// even though the listener could never start on them.
   #[test]
   fn empty_credentials_are_rejected() {
      assert!(validate_credentials("", "proj", "key").is_err());
      assert!(validate_credentials("1:123:android:abc", "", "key").is_err());
      assert!(validate_credentials("1:123:android:abc", "proj", "  ").is_err());
      validate_credentials("1:123:android:abc", "proj", "key").unwrap();
   }

   /// Two overlapping failed registrations: the first cannot retire the install
   /// while the second's row exists, so the second must retire it
   /// unconditionally rather than on whether it was the one that enrolled.
   #[tokio::test]
   async fn overlapping_rollbacks_leave_no_empty_install() {
      let db = test_db().await;
      let secret = InstallSecret::from("s1");
      let limiters = Limiters::generous();
      let install = InstallId::try_from(INSTALL_A).unwrap();
      let first_app = AppId::trusted("com.first");
      let second_app = AppId::trusted("com.second");

      for app_id in [&first_app, &second_app] {
         let identity =
            authenticate_register(&db, Some(INSTALL_A), Some(&secret), &limiters.limits())
               .await
               .unwrap()
               .claim(&db)
               .await
               .unwrap();
         let mut registration = reg(INSTALL_A, app_id.as_ref(), "");
         registration.secret_hash = identity.secret_hash;
         db.save_registration(&registration, "gen").await.unwrap();
      }

      // The first rollback is a no-op on the install, the second retires it.
      db.roll_back_registration(&install, &first_app, "gen")
         .await
         .unwrap();
      assert!(db.installation_hash(&install).await.unwrap().is_some());
      db.roll_back_registration(&install, &second_app, "gen")
         .await
         .unwrap();
      assert!(db.installation_hash(&install).await.unwrap().is_none());
   }

   /// A wrong secret must be refused before the write, not by it.
   #[tokio::test]
   async fn a_wrong_secret_is_rejected_without_touching_the_row() {
      let db = test_db().await;
      let limiters = Limiters::generous();
      let good = InstallSecret::from("s1");
      authorize_register(
         &db,
         &AppId::trusted("com.app"),
         Some(INSTALL_A),
         Some(&good),
         &limiters.limits(),
      )
      .await
      .unwrap();
      let stored = db
         .installation_hash(&InstallId::try_from(INSTALL_A).unwrap())
         .await
         .unwrap();

      let rejection = authenticate_register(
         &db,
         Some(INSTALL_A),
         Some(&InstallSecret::from("s2")),
         &limiters.limits(),
      )
      .await
      .err()
      .map(|(status, _)| status);
      assert_eq!(rejection, Some(StatusCode::UNAUTHORIZED));
      assert_eq!(
         db.installation_hash(&InstallId::try_from(INSTALL_A).unwrap())
            .await
            .unwrap(),
         stored
      );
   }

   /// Spending the shared budget must not lock out a registered tenant.
   #[tokio::test]
   async fn exhausted_enrollment_still_serves_existing_installs() {
      let db = test_db().await;
      let app_id = AppId::trusted("com.app");
      let secret = InstallSecret::from("s1");
      let limiters = Limiters {
         enrollment:  RateLimiter::new(1, Duration::from_secs(3600)),
         per_install: RateLimiter::new(1000, Duration::from_secs(1)),
      };

      authorize_register(
         &db,
         &app_id,
         Some(INSTALL_A),
         Some(&secret),
         &limiters.limits(),
      )
      .await
      .unwrap();

      let err = authorize_register(
         &db,
         &app_id,
         Some(INSTALL_B),
         Some(&secret),
         &limiters.limits(),
      )
      .await
      .unwrap_err();
      assert_eq!(err.0, StatusCode::TOO_MANY_REQUESTS);
      assert!(
         db.installation_hash(&InstallId::try_from(INSTALL_B).unwrap())
            .await
            .unwrap()
            .is_none()
      );

      authorize_register(
         &db,
         &app_id,
         Some(INSTALL_A),
         Some(&secret),
         &limiters.limits(),
      )
      .await
      .unwrap();
   }

   #[tokio::test]
   async fn a_single_install_can_be_rate_limited() {
      let db = test_db().await;
      let app_id = AppId::trusted("com.app");
      let secret = InstallSecret::from("s1");
      let limiters = Limiters {
         enrollment:  RateLimiter::new(1000, Duration::from_secs(1)),
         per_install: RateLimiter::new(1, Duration::from_secs(3600)),
      };

      // Enrolling does not spend the per-install budget.
      authorize_register(
         &db,
         &app_id,
         Some(INSTALL_A),
         Some(&secret),
         &limiters.limits(),
      )
      .await
      .unwrap();
      authorize_register(
         &db,
         &app_id,
         Some(INSTALL_A),
         Some(&secret),
         &limiters.limits(),
      )
      .await
      .unwrap();
      let err = authorize_register(
         &db,
         &app_id,
         Some(INSTALL_A),
         Some(&secret),
         &limiters.limits(),
      )
      .await
      .unwrap_err();
      assert_eq!(err.0, StatusCode::TOO_MANY_REQUESTS);
   }
}
