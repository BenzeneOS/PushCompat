use reqwest::Client;
use serde::{
   Deserialize,
   Serialize,
};
use tokio::{
   net::TcpStream,
   time::{
      Duration,
      sleep,
   },
};
use tokio_rustls::client::TlsStream;

use crate::{
   Error,
   FcmCredentials,
   MessageStream,
   gcm::{
      DeviceSessionState,
      FirebaseConfig,
      FirebaseRegistrationParams,
   },
};

#[derive(Clone, Debug)]
pub struct DeviceSession {
   state: DeviceSessionState,
}

impl DeviceSession {
   /// Creates a fresh device session.
   ///
   /// # Errors
   ///
   /// Returns an error when Google check-in fails.
   pub async fn fresh(http: &Client) -> Result<Self, Error> {
      Ok(Self {
         state: DeviceSessionState::checkin(http).await?,
      })
   }

   #[must_use]
   pub const fn restore(state: DeviceSessionState) -> Self {
      Self { state }
   }

   #[must_use]
   pub fn state(&self) -> DeviceSessionState {
      self.state.clone()
   }

   #[must_use]
   pub fn into_state(self) -> DeviceSessionState {
      self.state
   }

   /// Refreshes the device session through Google check-in.
   ///
   /// # Errors
   ///
   /// Returns an error when Google check-in fails.
   pub async fn refresh(&mut self, http: &Client) -> Result<(), Error> {
      self.state = self.state.refresh(http).await?;
      Ok(())
   }

   /// Connects the device session to MCS.
   ///
   /// # Errors
   ///
   /// Returns an error when the MCS connection cannot be established.
   pub async fn connect(
      &self,
      persistent_ids: Vec<String>,
   ) -> Result<MessageStream<TlsStream<TcpStream>>, Error> {
      let connection = self.state.connect(persistent_ids).await?;
      Ok(MessageStream::new(connection.0))
   }

   #[must_use]
   pub const fn android_id(&self) -> i64 {
      self.state.android_id
   }

   /// Decrypts an encrypted FCM payload.
   ///
   /// # Errors
   ///
   /// Returns an error when the payload or session keys are invalid.
   pub fn decrypt(&self, encrypted_base64: &str) -> Result<Vec<u8>, Error> {
      self.state.decrypt(encrypted_base64)
   }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppRegistrationState {
   pub fcm_token:   String,
   pub credentials: FcmCredentials,
}

#[derive(Clone, Debug)]
pub struct AppRegistration {
   state: AppRegistrationState,
}

impl AppRegistration {
   /// Registers an app with Firebase Cloud Messaging.
   ///
   /// # Errors
   ///
   /// Returns an error when Firebase installation or FCM registration fails.
   pub async fn register(
      http: &Client,
      device: &DeviceSession,
      credentials: FcmCredentials,
   ) -> Result<Self, Error> {
      sleep(Duration::from_millis(500)).await;

      let firebase_config = FirebaseConfig {
         project_id: credentials.project_id.clone(),
         api_key:    credentials.api_key.clone(),
         app_id:     credentials.app_id.clone(),
      };
      let firebase_installation = DeviceSessionState::register_firebase_installation(
         http,
         &firebase_config,
         &credentials.package_name,
         credentials.cert_sha1.as_deref().unwrap_or(""),
      )
      .await?;
      let gcm_token = device
         .state
         .register(http, FirebaseRegistrationParams {
            sender_id:             &credentials.sender_id,
            package_name:          &credentials.package_name,
            cert_sha1:             credentials.cert_sha1.as_deref(),
            app_version:           credentials.app_version,
            app_version_name:      credentials.app_version_name.as_deref(),
            target_sdk:            credentials.target_sdk,
            firebase_config:       Some(&firebase_config),
            firebase_installation: Some(&firebase_installation),
         })
         .await?;

      Ok(Self {
         state: AppRegistrationState {
            fcm_token: gcm_token.token,
            credentials,
         },
      })
   }

   #[must_use]
   pub const fn restore(state: AppRegistrationState) -> Self {
      Self { state }
   }

   #[must_use]
   pub fn state(&self) -> AppRegistrationState {
      self.state.clone()
   }

   #[must_use]
   pub fn into_state(self) -> AppRegistrationState {
      self.state
   }

   #[must_use]
   pub fn fcm_token(&self) -> &str {
      &self.state.fcm_token
   }

   #[must_use]
   pub const fn credentials(&self) -> &FcmCredentials {
      &self.state.credentials
   }
}
