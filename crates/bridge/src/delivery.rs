use std::sync::Arc;

use anyhow::Result;

use crate::{
   db::Database,
   socket::SocketHub,
   types::{
      AppId,
      ConnectorToken,
      InstallId,
      MessageId,
   },
};

/// Persists a message and wakes whichever socket is attached for the install.
///
/// Everything leaves over the client's own websocket, so there is no outbound
/// HTTP for a registration to steer at an internal address.
pub struct DeliveryManager {
   db:         Arc<Database>,
   socket_hub: Arc<SocketHub>,
}

impl DeliveryManager {
   pub const fn new(db: Arc<Database>, socket_hub: Arc<SocketHub>) -> Self {
      Self { db, socket_hub }
   }

   pub async fn enqueue_fcm(
      &self,
      install_id: &InstallId,
      app_id: &AppId,
      persistent_id: Option<&str>,
      payload: &[u8],
   ) -> Result<Option<MessageId>> {
      let message_id = self
         .db
         .enqueue_fcm_message(install_id, app_id, persistent_id, payload)
         .await?;

      if message_id.is_some() {
         self.socket_hub.wake(install_id.as_ref());
      }

      Ok(message_id)
   }

   pub async fn enqueue_unified_push(
      &self,
      install_id: &InstallId,
      app_id: &AppId,
      connector_token: &ConnectorToken,
      payload: &[u8],
   ) -> Result<MessageId> {
      let message_id = self
         .db
         .enqueue_unified_push_message(install_id, app_id, connector_token, payload)
         .await?;
      self.socket_hub.wake(install_id.as_ref());
      Ok(message_id)
   }
}
