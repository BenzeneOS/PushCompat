use std::{
   error::Error,
   fmt,
   ops::Deref,
};

use rusqlite::types::{
   FromSql,
   FromSqlError,
   FromSqlResult,
   ToSql,
   ToSqlOutput,
   Value,
   ValueRef,
};
use serde::{
   Deserialize,
   Deserializer,
   Serialize,
   de::Error as _,
};

const MIN_INSTALL_ID_LEN: usize = 16;
const MAX_INSTALL_ID_LEN: usize = 64;
const MAX_CONNECTOR_TOKEN_LEN: usize = 100;

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize)]
#[serde(transparent)]
pub struct InstallId(String);

impl TryFrom<&str> for InstallId {
   type Error = InvalidInstallId;

   fn try_from(value: &str) -> Result<Self, Self::Error> {
      Self::try_from(value.to_owned())
   }
}

impl TryFrom<String> for InstallId {
   type Error = InvalidInstallId;

   fn try_from(value: String) -> Result<Self, Self::Error> {
      if (MIN_INSTALL_ID_LEN..=MAX_INSTALL_ID_LEN).contains(&value.len())
         && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
      {
         Ok(Self(value))
      } else {
         Err(InvalidInstallId)
      }
   }
}

impl<'de> Deserialize<'de> for InstallId {
   fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
   where
      D: Deserializer<'de>,
   {
      Self::try_from(String::deserialize(deserializer)?).map_err(D::Error::custom)
   }

   fn deserialize_in_place<D>(deserializer: D, place: &mut Self) -> Result<(), D::Error>
   where
      D: Deserializer<'de>,
   {
      *place = Self::deserialize(deserializer)?;
      Ok(())
   }
}

impl AsRef<str> for InstallId {
   fn as_ref(&self) -> &str {
      &self.0
   }
}

impl fmt::Display for InstallId {
   fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      f.write_str(&self.0)
   }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InvalidInstallId;

impl fmt::Display for InvalidInstallId {
   fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      f.write_str("invalid install_id")
   }
}

impl Error for InvalidInstallId {
   fn source(&self) -> Option<&(dyn Error + 'static)> {
      None
   }
   fn description(&self) -> &'static str {
      "invalid install_id"
   }
   fn cause(&self) -> Option<&dyn Error> {
      None
   }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize)]
#[serde(transparent)]
pub struct AppId(String);

/// Android's own package-name ceiling. Request input has to be bounded or a
/// quota counted in rows says nothing about bytes on disk.
const MAX_APP_ID_LEN: usize = 255;

impl TryFrom<&str> for AppId {
   type Error = InvalidAppId;

   fn try_from(value: &str) -> Result<Self, Self::Error> {
      if value.is_empty() || value.len() > MAX_APP_ID_LEN {
         Err(InvalidAppId)
      } else {
         Ok(Self(value.to_owned()))
      }
   }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InvalidAppId;

impl fmt::Display for InvalidAppId {
   fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      f.write_str("invalid app_id")
   }
}

impl Error for InvalidAppId {
   fn source(&self) -> Option<&(dyn Error + 'static)> {
      None
   }
   fn description(&self) -> &'static str {
      "invalid app_id"
   }
   fn cause(&self) -> Option<&dyn Error> {
      None
   }
}

impl AppId {
   /// Bypasses [`TryFrom`], for values that did not come from a request.
   pub fn trusted(value: &str) -> Self {
      Self(value.to_owned())
   }
}

impl<'de> Deserialize<'de> for AppId {
   fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
   where
      D: Deserializer<'de>,
   {
      Self::try_from(String::deserialize(deserializer)?.as_str()).map_err(D::Error::custom)
   }

   fn deserialize_in_place<D>(deserializer: D, place: &mut Self) -> Result<(), D::Error>
   where
      D: Deserializer<'de>,
   {
      *place = Self::deserialize(deserializer)?;
      Ok(())
   }
}

impl AsRef<str> for AppId {
   fn as_ref(&self) -> &str {
      &self.0
   }
}

impl Deref for AppId {
   type Target = str;

   fn deref(&self) -> &Self::Target {
      self.as_ref()
   }
}

impl fmt::Display for AppId {
   fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      f.write_str(&self.0)
   }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize)]
#[serde(transparent)]
pub struct ConnectorToken(String);

impl TryFrom<&str> for ConnectorToken {
   type Error = InvalidConnectorToken;

   fn try_from(value: &str) -> Result<Self, Self::Error> {
      Self::try_from(value.to_owned())
   }
}

impl TryFrom<String> for ConnectorToken {
   type Error = InvalidConnectorToken;

   fn try_from(value: String) -> Result<Self, Self::Error> {
      if value.is_empty() || value.len() > MAX_CONNECTOR_TOKEN_LEN {
         Err(InvalidConnectorToken)
      } else {
         Ok(Self(value))
      }
   }
}

impl<'de> Deserialize<'de> for ConnectorToken {
   fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
   where
      D: Deserializer<'de>,
   {
      Self::try_from(String::deserialize(deserializer)?).map_err(D::Error::custom)
   }

   fn deserialize_in_place<D>(deserializer: D, place: &mut Self) -> Result<(), D::Error>
   where
      D: Deserializer<'de>,
   {
      *place = Self::deserialize(deserializer)?;
      Ok(())
   }
}

impl AsRef<str> for ConnectorToken {
   fn as_ref(&self) -> &str {
      &self.0
   }
}

impl Deref for ConnectorToken {
   type Target = str;

   fn deref(&self) -> &Self::Target {
      self.as_ref()
   }
}

impl fmt::Display for ConnectorToken {
   fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      f.write_str(&self.0)
   }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InvalidConnectorToken;

impl fmt::Display for InvalidConnectorToken {
   fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      f.write_str("invalid connector token")
   }
}

impl Error for InvalidConnectorToken {
   fn source(&self) -> Option<&(dyn Error + 'static)> {
      None
   }
   fn description(&self) -> &'static str {
      "invalid connector token"
   }
   fn cause(&self) -> Option<&dyn Error> {
      None
   }
}

pub struct InstallSecret(String);

impl InstallSecret {
   pub fn expose(&self) -> &str {
      &self.0
   }
}

impl From<&str> for InstallSecret {
   fn from(value: &str) -> Self {
      Self(value.to_owned())
   }
}

impl From<String> for InstallSecret {
   fn from(value: String) -> Self {
      Self(value)
   }
}

#[derive(Clone)]
pub struct PayloadKey([u8; 32]);

impl PayloadKey {
   pub const fn new(value: [u8; 32]) -> Self {
      Self(value)
   }

   pub const fn expose(&self) -> &[u8; 32] {
      &self.0
   }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum MessageKind {
   #[serde(rename = "fcm")]
   Fcm,
   #[serde(rename = "unified_push")]
   UnifiedPush,
}

impl TryFrom<&str> for MessageKind {
   type Error = InvalidMessageKind;

   fn try_from(value: &str) -> Result<Self, Self::Error> {
      match value {
         "fcm" => Ok(Self::Fcm),
         "unified_push" => Ok(Self::UnifiedPush),
         _ => Err(InvalidMessageKind),
      }
   }
}

impl AsRef<str> for MessageKind {
   fn as_ref(&self) -> &str {
      match self {
         Self::Fcm => "fcm",
         Self::UnifiedPush => "unified_push",
      }
   }
}

impl Deref for MessageKind {
   type Target = str;

   fn deref(&self) -> &Self::Target {
      self.as_ref()
   }
}

impl fmt::Display for MessageKind {
   fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      f.write_str(self.as_ref())
   }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InvalidMessageKind;

impl fmt::Display for InvalidMessageKind {
   fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      f.write_str("invalid message kind")
   }
}

impl Error for InvalidMessageKind {
   fn source(&self) -> Option<&(dyn Error + 'static)> {
      None
   }
   fn description(&self) -> &'static str {
      "invalid message kind"
   }
   fn cause(&self) -> Option<&dyn Error> {
      None
   }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(transparent)]
pub struct MessageId(i64);

impl MessageId {
   pub const fn new(value: i64) -> Self {
      Self(value)
   }

   pub const fn get(self) -> i64 {
      self.0
   }
}

impl fmt::Display for MessageId {
   fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      self.0.fmt(f)
   }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(transparent)]
pub struct Cursor(i64);

impl Cursor {
   pub fn clamp(self, maximum: MessageId) -> Self {
      Self(self.0.clamp(0, maximum.get()))
   }

   pub const fn get(self) -> i64 {
      self.0
   }
}

impl From<MessageId> for Cursor {
   fn from(id: MessageId) -> Self {
      Self(id.get())
   }
}

impl fmt::Display for Cursor {
   fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      self.0.fmt(f)
   }
}

macro_rules! impl_text_sql {
   ($type:ty) => {
      impl ToSql for $type {
         fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
            ToSql::to_sql(self.as_ref())
         }
      }

      impl FromSql for $type {
         fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
            let value = value.as_str()?;
            Self::try_from(value).map_err(|error| FromSqlError::Other(Box::new(error)))
         }
      }
   };
}

impl_text_sql!(InstallId);
impl_text_sql!(AppId);
impl_text_sql!(ConnectorToken);
impl_text_sql!(MessageKind);

impl ToSql for InstallSecret {
   fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
      ToSql::to_sql(self.expose())
   }
}

impl FromSql for InstallSecret {
   fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
      Ok(Self::from(value.as_str()?))
   }
}

impl ToSql for MessageId {
   fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
      Ok(ToSqlOutput::Owned(Value::Integer(self.get())))
   }
}

impl FromSql for MessageId {
   fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
      <i64 as FromSql>::column_result(value).map(Self::new)
   }
}

impl ToSql for Cursor {
   fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
      Ok(ToSqlOutput::Owned(Value::Integer(self.get())))
   }
}

impl FromSql for Cursor {
   fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
      <i64 as FromSql>::column_result(value).map(Self)
   }
}

#[cfg(test)]
mod tests {
   use super::*;

   /// A row quota bounds rows, not bytes, so request-supplied identifiers have
   /// to be length-limited on their own.
   #[test]
   fn app_id_from_a_request_is_bounded() {
      AppId::try_from("com.example.app").unwrap();
      AppId::try_from("").unwrap_err();
      AppId::try_from("x".repeat(MAX_APP_ID_LEN).as_str()).unwrap();
      AppId::try_from("x".repeat(MAX_APP_ID_LEN + 1).as_str()).unwrap_err();
   }
}
