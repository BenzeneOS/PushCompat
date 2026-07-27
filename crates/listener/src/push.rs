use std::{
   future::Future as _,
   io,
   ops::{
      Deref,
      DerefMut,
   },
   pin::Pin,
   str,
   task::{
      Context,
      Poll,
   },
   time::Duration,
};

use bytes::{
   Bytes,
   BytesMut,
};
use pin_project_lite::pin_project;
use quick_protobuf::{
   BytesReader,
   MessageRead as _,
   MessageWrite,
   Writer,
};
use tokio::{
   io::{
      AsyncRead,
      AsyncReadExt as _,
      AsyncWrite,
      AsyncWriteExt as _,
   },
   pin,
   time::timeout,
};
use tokio_stream::Stream;

use crate::{
   Error,
   mcs::{
      DataMessageStanza,
      Extension,
      HeartbeatAck,
      HeartbeatPing,
      IqStanza,
      LoginResponse,
      mod_IqStanza::IqType,
   },
};

const MAX_MCS_FRAME_BYTES: usize = 4 * 1024 * 1024;

#[derive(PartialEq, Eq, Debug)]
pub enum MessageTag {
   HeartbeatPing = 0,
   HeartbeatAck,
   LoginRequest,
   LoginResponse,
   Close,
   MessageStanza,
   PresenceStanza,
   IqStanza,
   DataMessageStanza,
   BatchPresenceStanza,
   StreamErrorStanza,
   HttpRequest,
   HttpResponse,
   BindAccountRequest,
   BindAccountResponse,
   TalkMetadata,
}

impl TryFrom<u8> for MessageTag {
   type Error = u8;

   fn try_from(value: u8) -> Result<Self, Self::Error> {
      match value {
         0 => Ok(Self::HeartbeatPing),
         1 => Ok(Self::HeartbeatAck),
         2 => Ok(Self::LoginRequest),
         3 => Ok(Self::LoginResponse),
         4 => Ok(Self::Close),
         5 => Ok(Self::MessageStanza),
         6 => Ok(Self::PresenceStanza),
         7 => Ok(Self::IqStanza),
         8 => Ok(Self::DataMessageStanza),
         9 => Ok(Self::BatchPresenceStanza),
         10 => Ok(Self::StreamErrorStanza),
         11 => Ok(Self::HttpRequest),
         12 => Ok(Self::HttpResponse),
         13 => Ok(Self::BindAccountRequest),
         14 => Ok(Self::BindAccountResponse),
         15 => Ok(Self::TalkMetadata),
         _ => Err(value),
      }
   }
}

pub enum Message {
   HeartbeatPing,
   HeartbeatAck,
   Data(DataMessage),
   Other(u8, Bytes),
}

pub const MCS_HEARTBEAT_INTERVAL: Duration = Duration::from_mins(15);
pub const MCS_IDLE_TIMEOUT: Duration = Duration::from_mins(75);
pub const MCS_WRITE_TIMEOUT: Duration = Duration::from_secs(30);

/// A dead socket can absorb writes into its send buffer until the TCP
/// retransmission timeout fires, which parks select arms the stop channel
/// cannot preempt.
///
/// # Errors
///
/// Returns an error when the write fails or exceeds the deadline.
pub async fn write_all_with_deadline<W>(writer: &mut W, bytes: &[u8]) -> io::Result<()>
where
   W: AsyncWrite + Unpin,
{
   timeout(MCS_WRITE_TIMEOUT, writer.write_all(bytes))
      .await
      .unwrap_or_else(|_| {
         Err(io::Error::new(
            io::ErrorKind::TimedOut,
            "MCS write deadline exceeded",
         ))
      })
}

#[derive(Debug)]
pub struct LoginResponseInfo {
   pub id:                      String,
   pub error_code:              Option<i32>,
   pub error_message:           Option<String>,
   pub error_type:              Option<String>,
   pub stream_id:               Option<i32>,
   pub last_stream_id_received: Option<i32>,
   pub server_timestamp:        Option<i64>,
}

/// Decode an MCS login response.
///
/// # Errors
///
/// Returns an error when the response is not valid protobuf.
pub fn decode_login_response(bytes: &[u8]) -> Result<LoginResponseInfo, Error> {
   let mut reader = BytesReader::from_bytes(bytes);
   let response = LoginResponse::from_reader(&mut reader, bytes)
      .map_err(|error| Error::ProtobufDecode("MCS login response", error))?;
   let (error_code, error_message, error_type) =
      response.error.map_or((None, None, None), |error| {
         (Some(error.code), error.message, error.type_pb)
      });
   Ok(LoginResponseInfo {
      id: response.id,
      error_code,
      error_message,
      error_type,
      stream_id: response.stream_id,
      last_stream_id_received: response.last_stream_id_received,
      server_timestamp: response.server_timestamp,
   })
}

/// A data message received from FCM.
pub struct DataMessage {
   /// Raw message data (typically JSON for FCM).
   pub raw_data:      Option<Vec<u8>>,
   /// Persistent ID for acknowledging receipt.
   pub persistent_id: Option<String>,
   /// App data key-value pairs.
   pub app_data:      Vec<(String, String)>,
   /// Source of the message (sender).
   pub from:          Option<String>,
   /// Package name used to demultiplex messages across app registrations.
   pub category:      Option<String>,
   /// Sender-assigned message ID. Becomes `google.message_id` in the Android
   /// intent. The Firebase SDK discards messages that arrive without it.
   pub id:            Option<String>,
   /// Collapse key, delivered as the `collapse_key` extra.
   pub collapse_key:  Option<String>,
}

impl DataMessage {
   fn decode(bytes: &[u8]) -> Result<Self, Error> {
      let mut reader = BytesReader::from_bytes(bytes);
      let message = DataMessageStanza::from_reader(&mut reader, bytes)
         .map_err(|error| Error::ProtobufDecode("FCM data message", error))?;

      let app_data = message
         .app_data
         .into_iter()
         .map(|field| (field.key, field.value))
         .collect::<Vec<(String, String)>>();

      Ok(Self {
         raw_data: message.raw_data,
         persistent_id: message.persistent_id,
         app_data,
         from: if message.from.is_empty() {
            None
         } else {
            Some(message.from)
         },
         category: if message.category.is_empty() {
            None
         } else {
            Some(message.category)
         },
         id: message.id,
         collapse_key: message.token,
      })
   }

   /// Get the message payload as bytes (if present).
   #[must_use]
   pub fn payload(&self) -> Option<&[u8]> {
      self.raw_data.as_deref()
   }

   /// Try to parse the payload as UTF-8 string.
   #[must_use]
   pub fn payload_str(&self) -> Option<&str> {
      let data = self.raw_data.as_ref()?;
      str::from_utf8(data).ok()
   }

   #[must_use]
   pub fn package_name(&self) -> Option<&str> {
      self.category.as_deref()
   }

   /// Get an `app_data` value by key.
   #[must_use]
   pub fn get_app_data(&self, key: &str) -> Option<&str> {
      self
         .app_data
         .iter()
         .find(|(entry_key, _)| entry_key == key)
         .map(|(_, value)| value.as_str())
   }
}

pin_project! {
    pub struct MessageStream<T> {
        #[pin]
        inner: T,
        bytes_required: usize,
        receive_buffer: BytesMut,
        last_stream_id_received: i32,
    }
}

impl<T> MessageStream<T> {
   pub fn new(inner: T) -> Self {
      Self {
         inner,
         bytes_required: 2,
         receive_buffer: BytesMut::with_capacity(1024),
         last_stream_id_received: 0,
      }
   }

   /// MCS omits stream IDs on the wire, so every decoded frame advances this
   /// counter.
   pub const fn last_stream_id_received(&self) -> i32 {
      self.last_stream_id_received
   }

   /// Decode the MCS varint32 frame length, or return `None` until it is
   /// complete.
   fn try_read_varint<'a>(
      mut bytes: impl Iterator<Item = &'a u8>,
   ) -> Result<Option<(usize, usize)>, Error> {
      let mut result = 0_usize;
      for index in 0_u32..5 {
         let Some(byte) = bytes.next().copied() else {
            return Ok(None);
         };
         let value_part = usize::from(byte & 0x7F);
         let shift = index.checked_mul(7).ok_or(Error::DependencyFailure(
            "MCS stream",
            "frame length overflowed",
         ))?;
         let shifted = value_part
            .checked_shl(shift)
            .ok_or(Error::DependencyFailure(
               "MCS stream",
               "frame length overflowed",
            ))?;
         result = result.checked_add(shifted).ok_or(Error::DependencyFailure(
            "MCS stream",
            "frame length overflowed",
         ))?;

         if byte & 0x80 == 0 {
            if result > MAX_MCS_FRAME_BYTES {
               return Err(Error::DependencyFailure("MCS stream", "frame is too large"));
            }
            let offset = usize::try_from(index)
               .ok()
               .and_then(|index| index.checked_add(2))
               .ok_or(Error::DependencyFailure(
                  "MCS stream",
                  "frame offset overflowed",
               ))?;
            return Ok(Some((result, offset)));
         }
      }
      Err(Error::DependencyFailure(
         "MCS stream",
         "frame length varint is invalid",
      ))
   }
}

impl<T> Stream for MessageStream<T>
where
   T: AsyncRead + AsyncWrite + Unpin,
{
   type Item = Result<Message, Error>;

   fn size_hint(&self) -> (usize, Option<usize>) {
      (0, None)
   }

   fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
      use bytes::Buf as _;

      loop {
         let mut bytes = self.receive_buffer.iter();
         if let Some(first_tag_value) = bytes.next() {
            let tag_value = *first_tag_value;
            let tag = MessageTag::try_from(tag_value);
            if matches!(tag, Ok(MessageTag::Close)) {
               self.bytes_required = 0;
               self.receive_buffer.clear();
               return Poll::Ready(None);
            }

            match Self::try_read_varint(bytes) {
               Ok(Some((size, offset))) => {
                  let Some(bytes_required) = offset.checked_add(size) else {
                     self.bytes_required = 0;
                     self.receive_buffer.clear();
                     return Poll::Ready(Some(Err(Error::DependencyFailure(
                        "MCS stream",
                        "frame size overflowed",
                     ))));
                  };
                  if bytes_required <= self.receive_buffer.len() {
                     // The next frame can be smaller, so return to the minimum
                     // read size after consuming this one.
                     self.bytes_required = 2;

                     self.receive_buffer.advance(offset);
                     let bytes = self.receive_buffer.split_to(size);
                     let Some(stream_id) = self.last_stream_id_received.checked_add(1) else {
                        self.bytes_required = 0;
                        self.receive_buffer.clear();
                        return Poll::Ready(Some(Err(Error::DependencyFailure(
                           "MCS stream",
                           "stream id overflowed",
                        ))));
                     };
                     self.last_stream_id_received = stream_id;
                     return Poll::Ready(Some(Ok(match tag {
                        Ok(MessageTag::DataMessageStanza) => {
                           match DataMessage::decode(&bytes) {
                              Err(error) => return Poll::Ready(Some(Err(error))),
                              Ok(message) => Message::Data(message),
                           }
                        },
                        Ok(MessageTag::HeartbeatPing) => Message::HeartbeatPing,
                        Ok(MessageTag::HeartbeatAck) => Message::HeartbeatAck,
                        _ => Message::Other(tag_value, bytes.into()),
                     })));
                  }

                  let capacity = self.receive_buffer.capacity();
                  if bytes_required > capacity {
                     let Some(additional) = bytes_required.checked_sub(capacity) else {
                        self.bytes_required = 0;
                        self.receive_buffer.clear();
                        return Poll::Ready(Some(Err(Error::DependencyFailure(
                           "MCS stream",
                           "frame capacity calculation overflowed",
                        ))));
                     };
                     self.receive_buffer.reserve(additional);
                  }
                  self.bytes_required = bytes_required;
               },
               Ok(None) => {
                  let Some(bytes_required) = self.receive_buffer.len().checked_add(1) else {
                     self.bytes_required = 0;
                     self.receive_buffer.clear();
                     return Poll::Ready(Some(Err(Error::DependencyFailure(
                        "MCS stream",
                        "frame header size overflowed",
                     ))));
                  };
                  self.bytes_required = bytes_required;
               },
               Err(error) => {
                  self.bytes_required = 0;
                  self.receive_buffer.clear();
                  return Poll::Ready(Some(Err(error)));
               },
            }
         } else if self.bytes_required == 0 {
            return Poll::Ready(None);
         }

         loop {
            // insufficient data in the buffer, fill from inner
            let mut that = self.as_mut().project();
            let task = that.inner.read_buf(that.receive_buffer);
            pin!(task);
            match task.poll(cx) {
               Poll::Pending => return Poll::Pending,
               Poll::Ready(Err(error)) => {
                  // failfast
                  self.bytes_required = 0;
                  self.receive_buffer.clear();
                  return Poll::Ready(Some(Err(Error::Socket(error))));
               },
               Poll::Ready(Ok(0)) => {
                  // probably a broken pipe, which means whatever incomplete
                  // message we have buffered will just have to be chucked
                  self.bytes_required = 0;
                  self.receive_buffer.clear();
                  return Poll::Ready(None);
               },
               Poll::Ready(_) => {
                  if self.receive_buffer.len() >= self.bytes_required {
                     break;
                  }
               },
            }
         }
      }
   }
}

impl<T> Deref for MessageStream<T> {
   type Target = T;

   fn deref(&self) -> &Self::Target {
      &self.inner
   }
}

impl<T> DerefMut for MessageStream<T> {
   fn deref_mut(&mut self) -> &mut Self::Target {
      &mut self.inner
   }
}

fn encode_frame(tag: MessageTag, message: &impl MessageWrite) -> BytesMut {
   let Some(capacity) = message.get_size().checked_add(5) else {
      return BytesMut::new();
   };
   let mut bytes = Vec::new();
   if bytes.try_reserve(capacity).is_err() {
      return BytesMut::new();
   }
   let mut writer = Writer::new(&mut bytes);
   if writer
      .write_u8(tag as u8)
      .and_then(|()| writer.write_message(message))
      .is_err()
   {
      return BytesMut::new();
   }

   BytesMut::from(bytes.as_slice())
}

#[must_use]
pub fn new_heartbeat_ack(last_stream_id_received: i32) -> BytesMut {
   encode_frame(MessageTag::HeartbeatAck, &HeartbeatAck {
      last_stream_id_received: Some(last_stream_id_received),
      ..Default::default()
   })
}

#[must_use]
pub fn new_heartbeat_ping(last_stream_id_received: i32) -> BytesMut {
   encode_frame(MessageTag::HeartbeatPing, &HeartbeatPing {
      last_stream_id_received: Some(last_stream_id_received),
      ..Default::default()
   })
}

#[must_use]
pub fn new_stream_ack(last_stream_id_received: i32) -> BytesMut {
   encode_frame(MessageTag::IqStanza, &IqStanza {
      type_pb: IqType::SET,
      id: String::new(),
      extension: Some(Extension {
         id:   13,
         data: Vec::new(),
      }),
      last_stream_id_received: Some(last_stream_id_received),
      ..Default::default()
   })
}
