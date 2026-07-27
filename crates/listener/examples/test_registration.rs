//! Test the full FCM registration flow with Firebase Installations.

use std::{
   env,
   error::Error as StdError,
   net::{
      IpAddr,
      Ipv4Addr,
   },
};

use pushcompat_listener::{
   AppRegistration,
   DeviceSession,
   FcmCredentials,
};
use tracing::level_filters::LevelFilter;
fn required_env(key: &str) -> Result<String, String> {
   env::var(key).map_err(|_| format!("missing env var {key}"))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn StdError>> {
   let log_level = env::var("RUST_LOG")
      .ok()
      .and_then(|value| value.parse().ok())
      .unwrap_or(LevelFilter::INFO);
   tracing_subscriber::fmt().with_max_level(log_level).init();

   let http = pushcompat_listener::http_client_builder()
      .http1_only()
      .local_address(IpAddr::V4(Ipv4Addr::UNSPECIFIED))
      .pool_max_idle_per_host(0)
      .build()?;

   let creds = FcmCredentials {
      sender_id:        required_env("PUSHCOMPAT_SENDER_ID")?,
      api_key:          required_env("PUSHCOMPAT_API_KEY")?,
      app_id:           required_env("PUSHCOMPAT_APP_ID")?,
      project_id:       required_env("PUSHCOMPAT_PROJECT_ID")?,
      package_name:     required_env("PUSHCOMPAT_PACKAGE")?,
      cert_sha1:        env::var("PUSHCOMPAT_CERT_SHA1").ok(),
      app_version:      env::var("PUSHCOMPAT_APP_VERSION")
         .ok()
         .and_then(|value| value.parse().ok()),
      app_version_name: env::var("PUSHCOMPAT_APP_VERSION_NAME").ok(),
      target_sdk:       env::var("PUSHCOMPAT_TARGET_SDK")
         .ok()
         .and_then(|value| value.parse().ok()),
   };

   println!("=== Testing FCM Registration ===");
   println!("Package: {}", creds.package_name);
   println!("Sender ID: {}", creds.sender_id);
   println!();

   let device = DeviceSession::fresh(&http).await?;
   match AppRegistration::register(&http, &device, creds).await {
      Ok(registration) => {
         println!("✅ SUCCESS!");
         println!();
         println!("android_id: {}", device.android_id());
         let token = registration.fcm_token();
         println!("FCM Token: {}...", &token[..24.min(token.len())]);
         println!("Token length: {}", token.len());
      },
      Err(error) => {
         println!("❌ FAILED: {error}");
         return Err(error.into());
      },
   }

   Ok(())
}
