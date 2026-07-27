// Automatically generated rust module for 'mcs.proto' file
// Regenerate from the repository root: nix develop -c nu nix/regen-proto.nu

#![allow(non_snake_case, reason = "generated protobuf code")]
#![allow(non_upper_case_globals, reason = "generated protobuf code")]
#![allow(non_camel_case_types, reason = "generated protobuf code")]
#![allow(dead_code, reason = "generated protobuf code")]
#![allow(unused_imports, reason = "generated protobuf code")]
#![allow(unknown_lints, reason = "generated protobuf code")]
#![allow(clippy::all, reason = "generated protobuf code")]
#![cfg_attr(rustfmt, rustfmt_skip)]

use std::{fs::File, io::BufWriter, path::Path};


use quick_protobuf::{MessageInfo, MessageRead, MessageWrite, BytesReader, Writer, WriterBackend, Result};
use quick_protobuf::sizeofs::{sizeof_len, sizeof_varint};
use super::*;

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct HeartbeatPing {
    pub stream_id: Option<i32>,
    pub last_stream_id_received: Option<i32>,
    pub status: Option<i64>,
}

impl<'a> MessageRead<'a> for HeartbeatPing {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.stream_id = Some(r.read_int32(bytes)?),
                Ok(16) => msg.last_stream_id_received = Some(r.read_int32(bytes)?),
                Ok(24) => msg.status = Some(r.read_int64(bytes)?),
                Ok(tag) => { r.read_unknown(bytes, tag)?; }
                Err(error) => return Err(error),
            }
        }
        Ok(msg)
    }
}

impl MessageWrite for HeartbeatPing {
    fn get_size(&self) -> usize {
        0
        + self.stream_id.as_ref().map_or(0, |value| 1 + sizeof_varint(*(value) as u64))
        + self.last_stream_id_received.as_ref().map_or(0, |value| 1 + sizeof_varint(*(value) as u64))
        + self.status.as_ref().map_or(0, |value| 1 + sizeof_varint(*(value) as u64))
    }

    fn write_message<W>(&self, writer: &mut Writer<W>) -> Result<()>
    where
        W: WriterBackend,
    {
        if let Some(ref value) = self.stream_id { writer.write_with_tag(8, |writer| writer.write_int32(*value))?; }
        if let Some(ref value) = self.last_stream_id_received { writer.write_with_tag(16, |writer| writer.write_int32(*value))?; }
        if let Some(ref value) = self.status { writer.write_with_tag(24, |writer| writer.write_int64(*value))?; }
        Ok(())
    }

    fn write_file<P>(&self, p: P) -> Result<()>
    where
        P: AsRef<Path>,
    {
        let file = BufWriter::new(File::create(p)?);
        let mut writer = Writer::new(file);
        self.write_message(&mut writer)
    }
}

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct HeartbeatAck {
    pub stream_id: Option<i32>,
    pub last_stream_id_received: Option<i32>,
    pub status: Option<i64>,
}

impl<'a> MessageRead<'a> for HeartbeatAck {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.stream_id = Some(r.read_int32(bytes)?),
                Ok(16) => msg.last_stream_id_received = Some(r.read_int32(bytes)?),
                Ok(24) => msg.status = Some(r.read_int64(bytes)?),
                Ok(tag) => { r.read_unknown(bytes, tag)?; }
                Err(error) => return Err(error),
            }
        }
        Ok(msg)
    }
}

impl MessageWrite for HeartbeatAck {
    fn get_size(&self) -> usize {
        0
        + self.stream_id.as_ref().map_or(0, |value| 1 + sizeof_varint(*(value) as u64))
        + self.last_stream_id_received.as_ref().map_or(0, |value| 1 + sizeof_varint(*(value) as u64))
        + self.status.as_ref().map_or(0, |value| 1 + sizeof_varint(*(value) as u64))
    }

    fn write_message<W>(&self, writer: &mut Writer<W>) -> Result<()>
    where
        W: WriterBackend,
    {
        if let Some(ref value) = self.stream_id { writer.write_with_tag(8, |writer| writer.write_int32(*value))?; }
        if let Some(ref value) = self.last_stream_id_received { writer.write_with_tag(16, |writer| writer.write_int32(*value))?; }
        if let Some(ref value) = self.status { writer.write_with_tag(24, |writer| writer.write_int64(*value))?; }
        Ok(())
    }

    fn write_file<P>(&self, p: P) -> Result<()>
    where
        P: AsRef<Path>,
    {
        let file = BufWriter::new(File::create(p)?);
        let mut writer = Writer::new(file);
        self.write_message(&mut writer)
    }
}

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct ErrorInfo {
    pub code: i32,
    pub message: Option<String>,
    pub type_pb: Option<String>,
    pub extension: Option<Extension>,
}

impl<'a> MessageRead<'a> for ErrorInfo {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.code = r.read_int32(bytes)?,
                Ok(18) => msg.message = Some(r.read_string(bytes)?.to_owned()),
                Ok(26) => msg.type_pb = Some(r.read_string(bytes)?.to_owned()),
                Ok(34) => msg.extension = Some(r.read_message::<Extension>(bytes)?),
                Ok(tag) => { r.read_unknown(bytes, tag)?; }
                Err(error) => return Err(error),
            }
        }
        Ok(msg)
    }
}

impl MessageWrite for ErrorInfo {
    fn get_size(&self) -> usize {
        0
        + 1 + sizeof_varint(*(&self.code) as u64)
        + self.message.as_ref().map_or(0, |value| 1 + sizeof_len((value).len()))
        + self.type_pb.as_ref().map_or(0, |value| 1 + sizeof_len((value).len()))
        + self.extension.as_ref().map_or(0, |value| 1 + sizeof_len((value).get_size()))
    }

    fn write_message<W>(&self, writer: &mut Writer<W>) -> Result<()>
    where
        W: WriterBackend,
    {
        writer.write_with_tag(8, |writer| writer.write_int32(*&self.code))?;
        if let Some(ref value) = self.message { writer.write_with_tag(18, |writer| writer.write_string(&**value))?; }
        if let Some(ref value) = self.type_pb { writer.write_with_tag(26, |writer| writer.write_string(&**value))?; }
        if let Some(ref value) = self.extension { writer.write_with_tag(34, |writer| writer.write_message(value))?; }
        Ok(())
    }

    fn write_file<P>(&self, p: P) -> Result<()>
    where
        P: AsRef<Path>,
    {
        let file = BufWriter::new(File::create(p)?);
        let mut writer = Writer::new(file);
        self.write_message(&mut writer)
    }
}

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct Setting {
    pub name: String,
    pub value: String,
}

impl<'a> MessageRead<'a> for Setting {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => r.read_string(bytes)?.clone_into(&mut msg.name),
                Ok(18) => r.read_string(bytes)?.clone_into(&mut msg.value),
                Ok(tag) => { r.read_unknown(bytes, tag)?; }
                Err(error) => return Err(error),
            }
        }
        Ok(msg)
    }
}

impl MessageWrite for Setting {
    fn get_size(&self) -> usize {
        0
        + 1 + sizeof_len((&self.name).len())
        + 1 + sizeof_len((&self.value).len())
    }

    fn write_message<W>(&self, writer: &mut Writer<W>) -> Result<()>
    where
        W: WriterBackend,
    {
        writer.write_with_tag(10, |writer| writer.write_string(&**&self.name))?;
        writer.write_with_tag(18, |writer| writer.write_string(&**&self.value))?;
        Ok(())
    }

    fn write_file<P>(&self, p: P) -> Result<()>
    where
        P: AsRef<Path>,
    {
        let file = BufWriter::new(File::create(p)?);
        let mut writer = Writer::new(file);
        self.write_message(&mut writer)
    }
}

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct HeartbeatStat {
    pub ip: String,
    pub timeout: bool,
    pub interval_ms: i32,
}

impl<'a> MessageRead<'a> for HeartbeatStat {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => r.read_string(bytes)?.clone_into(&mut msg.ip),
                Ok(16) => msg.timeout = r.read_bool(bytes)?,
                Ok(24) => msg.interval_ms = r.read_int32(bytes)?,
                Ok(tag) => { r.read_unknown(bytes, tag)?; }
                Err(error) => return Err(error),
            }
        }
        Ok(msg)
    }
}

impl MessageWrite for HeartbeatStat {
    fn get_size(&self) -> usize {
        0
        + 1 + sizeof_len((&self.ip).len())
        + 1 + sizeof_varint(u64::from(self.timeout))
        + 1 + sizeof_varint(*(&self.interval_ms) as u64)
    }

    fn write_message<W>(&self, writer: &mut Writer<W>) -> Result<()>
    where
        W: WriterBackend,
    {
        writer.write_with_tag(10, |writer| writer.write_string(&**&self.ip))?;
        writer.write_with_tag(16, |writer| writer.write_bool(*&self.timeout))?;
        writer.write_with_tag(24, |writer| writer.write_int32(*&self.interval_ms))?;
        Ok(())
    }

    fn write_file<P>(&self, p: P) -> Result<()>
    where
        P: AsRef<Path>,
    {
        let file = BufWriter::new(File::create(p)?);
        let mut writer = Writer::new(file);
        self.write_message(&mut writer)
    }
}

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct HeartbeatConfig {
    pub upload_stat: Option<bool>,
    pub ip: Option<String>,
    pub interval_ms: Option<i32>,
}

impl<'a> MessageRead<'a> for HeartbeatConfig {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.upload_stat = Some(r.read_bool(bytes)?),
                Ok(18) => msg.ip = Some(r.read_string(bytes)?.to_owned()),
                Ok(24) => msg.interval_ms = Some(r.read_int32(bytes)?),
                Ok(tag) => { r.read_unknown(bytes, tag)?; }
                Err(error) => return Err(error),
            }
        }
        Ok(msg)
    }
}

impl MessageWrite for HeartbeatConfig {
    fn get_size(&self) -> usize {
        0
        + self.upload_stat.as_ref().map_or(0, |value| 1 + sizeof_varint(u64::from(*value)))
        + self.ip.as_ref().map_or(0, |value| 1 + sizeof_len((value).len()))
        + self.interval_ms.as_ref().map_or(0, |value| 1 + sizeof_varint(*(value) as u64))
    }

    fn write_message<W>(&self, writer: &mut Writer<W>) -> Result<()>
    where
        W: WriterBackend,
    {
        if let Some(ref value) = self.upload_stat { writer.write_with_tag(8, |writer| writer.write_bool(*value))?; }
        if let Some(ref value) = self.ip { writer.write_with_tag(18, |writer| writer.write_string(&**value))?; }
        if let Some(ref value) = self.interval_ms { writer.write_with_tag(24, |writer| writer.write_int32(*value))?; }
        Ok(())
    }

    fn write_file<P>(&self, p: P) -> Result<()>
    where
        P: AsRef<Path>,
    {
        let file = BufWriter::new(File::create(p)?);
        let mut writer = Writer::new(file);
        self.write_message(&mut writer)
    }
}

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct ClientEvent {
    pub type_pb: Option<mod_ClientEvent::Type>,
    pub number_discarded_events: Option<u32>,
    pub network_type: Option<i32>,
    pub time_connection_started_ms: Option<u64>,
    pub time_connection_ended_ms: Option<u64>,
    pub error_code: Option<i32>,
    pub time_connection_established_ms: Option<u64>,
}

impl<'a> MessageRead<'a> for ClientEvent {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.type_pb = Some(r.read_enum(bytes)?),
                Ok(800) => msg.number_discarded_events = Some(r.read_uint32(bytes)?),
                Ok(1600) => msg.network_type = Some(r.read_int32(bytes)?),
                Ok(1616) => msg.time_connection_started_ms = Some(r.read_uint64(bytes)?),
                Ok(1624) => msg.time_connection_ended_ms = Some(r.read_uint64(bytes)?),
                Ok(1632) => msg.error_code = Some(r.read_int32(bytes)?),
                Ok(2400) => msg.time_connection_established_ms = Some(r.read_uint64(bytes)?),
                Ok(tag) => { r.read_unknown(bytes, tag)?; }
                Err(error) => return Err(error),
            }
        }
        Ok(msg)
    }
}

impl MessageWrite for ClientEvent {
    fn get_size(&self) -> usize {
        0
        + self.type_pb.as_ref().map_or(0, |value| 1 + sizeof_varint(*(value) as u64))
        + self.number_discarded_events.as_ref().map_or(0, |value| 2 + sizeof_varint(u64::from(*value)))
        + self.network_type.as_ref().map_or(0, |value| 2 + sizeof_varint(*(value) as u64))
        + self.time_connection_started_ms.as_ref().map_or(0, |value| 2 + sizeof_varint(*(value) as u64))
        + self.time_connection_ended_ms.as_ref().map_or(0, |value| 2 + sizeof_varint(*(value) as u64))
        + self.error_code.as_ref().map_or(0, |value| 2 + sizeof_varint(*(value) as u64))
        + self.time_connection_established_ms.as_ref().map_or(0, |value| 2 + sizeof_varint(*(value) as u64))
    }

    fn write_message<W>(&self, writer: &mut Writer<W>) -> Result<()>
    where
        W: WriterBackend,
    {
        if let Some(ref value) = self.type_pb { writer.write_with_tag(8, |writer| writer.write_enum(*value as i32))?; }
        if let Some(ref value) = self.number_discarded_events { writer.write_with_tag(800, |writer| writer.write_uint32(*value))?; }
        if let Some(ref value) = self.network_type { writer.write_with_tag(1600, |writer| writer.write_int32(*value))?; }
        if let Some(ref value) = self.time_connection_started_ms { writer.write_with_tag(1616, |writer| writer.write_uint64(*value))?; }
        if let Some(ref value) = self.time_connection_ended_ms { writer.write_with_tag(1624, |writer| writer.write_uint64(*value))?; }
        if let Some(ref value) = self.error_code { writer.write_with_tag(1632, |writer| writer.write_int32(*value))?; }
        if let Some(ref value) = self.time_connection_established_ms { writer.write_with_tag(2400, |writer| writer.write_uint64(*value))?; }
        Ok(())
    }

    fn write_file<P>(&self, p: P) -> Result<()>
    where
        P: AsRef<Path>,
    {
        let file = BufWriter::new(File::create(p)?);
        let mut writer = Writer::new(file);
        self.write_message(&mut writer)
    }
}

pub mod mod_ClientEvent {


#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Type {
    UNKNOWN = 0,
    DISCARDED_EVENTS = 1,
    FAILED_CONNECTION = 2,
    SUCCESSFUL_CONNECTION = 3,
}

impl Default for Type {
    fn default() -> Self {
        Self::UNKNOWN
    }
}

impl From<i32> for Type {
    fn from(value: i32) -> Self {
        match value {
            0 => Self::UNKNOWN,
            1 => Self::DISCARDED_EVENTS,
            2 => Self::FAILED_CONNECTION,
            3 => Self::SUCCESSFUL_CONNECTION,
            _ => Self::default(),
        }
    }
}

impl<'a> From<&'a str> for Type {
    fn from(value: &'a str) -> Self {
        match value {
            "UNKNOWN" => Self::UNKNOWN,
            "DISCARDED_EVENTS" => Self::DISCARDED_EVENTS,
            "FAILED_CONNECTION" => Self::FAILED_CONNECTION,
            "SUCCESSFUL_CONNECTION" => Self::SUCCESSFUL_CONNECTION,
            _ => Self::default(),
        }
    }
}

}

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct LoginRequest {
    pub id: String,
    pub domain: String,
    pub user: String,
    pub resource: String,
    pub auth_token: String,
    pub device_id: Option<String>,
    pub last_rmq_id: Option<i64>,
    pub setting: Vec<Setting>,
    pub received_persistent_id: Vec<String>,
    pub adaptive_heartbeat: Option<bool>,
    pub heartbeat_stat: Option<HeartbeatStat>,
    pub use_rmq2: Option<bool>,
    pub account_id: Option<i64>,
    pub auth_service: Option<mod_LoginRequest::AuthService>,
    pub network_type: Option<i32>,
    pub status: Option<i64>,
    pub client_event: Vec<ClientEvent>,
}

impl<'a> MessageRead<'a> for LoginRequest {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => r.read_string(bytes)?.clone_into(&mut msg.id),
                Ok(18) => r.read_string(bytes)?.clone_into(&mut msg.domain),
                Ok(26) => r.read_string(bytes)?.clone_into(&mut msg.user),
                Ok(34) => r.read_string(bytes)?.clone_into(&mut msg.resource),
                Ok(42) => r.read_string(bytes)?.clone_into(&mut msg.auth_token),
                Ok(50) => msg.device_id = Some(r.read_string(bytes)?.to_owned()),
                Ok(56) => msg.last_rmq_id = Some(r.read_int64(bytes)?),
                Ok(66) => msg.setting.push(r.read_message::<Setting>(bytes)?),
                Ok(82) => msg.received_persistent_id.push(r.read_string(bytes)?.to_owned()),
                Ok(96) => msg.adaptive_heartbeat = Some(r.read_bool(bytes)?),
                Ok(106) => msg.heartbeat_stat = Some(r.read_message::<HeartbeatStat>(bytes)?),
                Ok(112) => msg.use_rmq2 = Some(r.read_bool(bytes)?),
                Ok(120) => msg.account_id = Some(r.read_int64(bytes)?),
                Ok(128) => msg.auth_service = Some(r.read_enum(bytes)?),
                Ok(136) => msg.network_type = Some(r.read_int32(bytes)?),
                Ok(144) => msg.status = Some(r.read_int64(bytes)?),
                Ok(178) => msg.client_event.push(r.read_message::<ClientEvent>(bytes)?),
                Ok(tag) => { r.read_unknown(bytes, tag)?; }
                Err(error) => return Err(error),
            }
        }
        Ok(msg)
    }
}

impl MessageWrite for LoginRequest {
    fn get_size(&self) -> usize {
        0
        + 1 + sizeof_len((&self.id).len())
        + 1 + sizeof_len((&self.domain).len())
        + 1 + sizeof_len((&self.user).len())
        + 1 + sizeof_len((&self.resource).len())
        + 1 + sizeof_len((&self.auth_token).len())
        + self.device_id.as_ref().map_or(0, |value| 1 + sizeof_len((value).len()))
        + self.last_rmq_id.as_ref().map_or(0, |value| 1 + sizeof_varint(*(value) as u64))
        + self.setting.iter().map(|value| 1 + sizeof_len((value).get_size())).sum::<usize>()
        + self.received_persistent_id.iter().map(|value| 1 + sizeof_len((value).len())).sum::<usize>()
        + self.adaptive_heartbeat.as_ref().map_or(0, |value| 1 + sizeof_varint(u64::from(*value)))
        + self.heartbeat_stat.as_ref().map_or(0, |value| 1 + sizeof_len((value).get_size()))
        + self.use_rmq2.as_ref().map_or(0, |value| 1 + sizeof_varint(u64::from(*value)))
        + self.account_id.as_ref().map_or(0, |value| 1 + sizeof_varint(*(value) as u64))
        + self.auth_service.as_ref().map_or(0, |value| 2 + sizeof_varint(*(value) as u64))
        + self.network_type.as_ref().map_or(0, |value| 2 + sizeof_varint(*(value) as u64))
        + self.status.as_ref().map_or(0, |value| 2 + sizeof_varint(*(value) as u64))
        + self.client_event.iter().map(|value| 2 + sizeof_len((value).get_size())).sum::<usize>()
    }

    fn write_message<W>(&self, writer: &mut Writer<W>) -> Result<()>
    where
        W: WriterBackend,
    {
        writer.write_with_tag(10, |writer| writer.write_string(&**&self.id))?;
        writer.write_with_tag(18, |writer| writer.write_string(&**&self.domain))?;
        writer.write_with_tag(26, |writer| writer.write_string(&**&self.user))?;
        writer.write_with_tag(34, |writer| writer.write_string(&**&self.resource))?;
        writer.write_with_tag(42, |writer| writer.write_string(&**&self.auth_token))?;
        if let Some(ref value) = self.device_id { writer.write_with_tag(50, |writer| writer.write_string(&**value))?; }
        if let Some(ref value) = self.last_rmq_id { writer.write_with_tag(56, |writer| writer.write_int64(*value))?; }
        for value in &self.setting { writer.write_with_tag(66, |writer| writer.write_message(value))?; }
        for value in &self.received_persistent_id { writer.write_with_tag(82, |writer| writer.write_string(&**value))?; }
        if let Some(ref value) = self.adaptive_heartbeat { writer.write_with_tag(96, |writer| writer.write_bool(*value))?; }
        if let Some(ref value) = self.heartbeat_stat { writer.write_with_tag(106, |writer| writer.write_message(value))?; }
        if let Some(ref value) = self.use_rmq2 { writer.write_with_tag(112, |writer| writer.write_bool(*value))?; }
        if let Some(ref value) = self.account_id { writer.write_with_tag(120, |writer| writer.write_int64(*value))?; }
        if let Some(ref value) = self.auth_service { writer.write_with_tag(128, |writer| writer.write_enum(*value as i32))?; }
        if let Some(ref value) = self.network_type { writer.write_with_tag(136, |writer| writer.write_int32(*value))?; }
        if let Some(ref value) = self.status { writer.write_with_tag(144, |writer| writer.write_int64(*value))?; }
        for value in &self.client_event { writer.write_with_tag(178, |writer| writer.write_message(value))?; }
        Ok(())
    }

    fn write_file<P>(&self, p: P) -> Result<()>
    where
        P: AsRef<Path>,
    {
        let file = BufWriter::new(File::create(p)?);
        let mut writer = Writer::new(file);
        self.write_message(&mut writer)
    }
}

pub mod mod_LoginRequest {


#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum AuthService {
    ANDROID_ID = 2,
}

impl Default for AuthService {
    fn default() -> Self {
        Self::ANDROID_ID
    }
}

impl From<i32> for AuthService {
    fn from(value: i32) -> Self {
        match value {
            2 => Self::ANDROID_ID,
            _ => Self::default(),
        }
    }
}

impl<'a> From<&'a str> for AuthService {
    fn from(value: &'a str) -> Self {
        match value {
            "ANDROID_ID" => Self::ANDROID_ID,
            _ => Self::default(),
        }
    }
}

}

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct LoginResponse {
    pub id: String,
    pub jid: Option<String>,
    pub error: Option<ErrorInfo>,
    pub setting: Vec<Setting>,
    pub stream_id: Option<i32>,
    pub last_stream_id_received: Option<i32>,
    pub heartbeat_config: Option<HeartbeatConfig>,
    pub server_timestamp: Option<i64>,
}

impl<'a> MessageRead<'a> for LoginResponse {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => r.read_string(bytes)?.clone_into(&mut msg.id),
                Ok(18) => msg.jid = Some(r.read_string(bytes)?.to_owned()),
                Ok(26) => msg.error = Some(r.read_message::<ErrorInfo>(bytes)?),
                Ok(34) => msg.setting.push(r.read_message::<Setting>(bytes)?),
                Ok(40) => msg.stream_id = Some(r.read_int32(bytes)?),
                Ok(48) => msg.last_stream_id_received = Some(r.read_int32(bytes)?),
                Ok(58) => msg.heartbeat_config = Some(r.read_message::<HeartbeatConfig>(bytes)?),
                Ok(64) => msg.server_timestamp = Some(r.read_int64(bytes)?),
                Ok(tag) => { r.read_unknown(bytes, tag)?; }
                Err(error) => return Err(error),
            }
        }
        Ok(msg)
    }
}

impl MessageWrite for LoginResponse {
    fn get_size(&self) -> usize {
        0
        + 1 + sizeof_len((&self.id).len())
        + self.jid.as_ref().map_or(0, |value| 1 + sizeof_len((value).len()))
        + self.error.as_ref().map_or(0, |value| 1 + sizeof_len((value).get_size()))
        + self.setting.iter().map(|value| 1 + sizeof_len((value).get_size())).sum::<usize>()
        + self.stream_id.as_ref().map_or(0, |value| 1 + sizeof_varint(*(value) as u64))
        + self.last_stream_id_received.as_ref().map_or(0, |value| 1 + sizeof_varint(*(value) as u64))
        + self.heartbeat_config.as_ref().map_or(0, |value| 1 + sizeof_len((value).get_size()))
        + self.server_timestamp.as_ref().map_or(0, |value| 1 + sizeof_varint(*(value) as u64))
    }

    fn write_message<W>(&self, writer: &mut Writer<W>) -> Result<()>
    where
        W: WriterBackend,
    {
        writer.write_with_tag(10, |writer| writer.write_string(&**&self.id))?;
        if let Some(ref value) = self.jid { writer.write_with_tag(18, |writer| writer.write_string(&**value))?; }
        if let Some(ref value) = self.error { writer.write_with_tag(26, |writer| writer.write_message(value))?; }
        for value in &self.setting { writer.write_with_tag(34, |writer| writer.write_message(value))?; }
        if let Some(ref value) = self.stream_id { writer.write_with_tag(40, |writer| writer.write_int32(*value))?; }
        if let Some(ref value) = self.last_stream_id_received { writer.write_with_tag(48, |writer| writer.write_int32(*value))?; }
        if let Some(ref value) = self.heartbeat_config { writer.write_with_tag(58, |writer| writer.write_message(value))?; }
        if let Some(ref value) = self.server_timestamp { writer.write_with_tag(64, |writer| writer.write_int64(*value))?; }
        Ok(())
    }

    fn write_file<P>(&self, p: P) -> Result<()>
    where
        P: AsRef<Path>,
    {
        let file = BufWriter::new(File::create(p)?);
        let mut writer = Writer::new(file);
        self.write_message(&mut writer)
    }
}

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct StreamErrorStanza {
    pub type_pb: String,
    pub text: Option<String>,
}

impl<'a> MessageRead<'a> for StreamErrorStanza {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => r.read_string(bytes)?.clone_into(&mut msg.type_pb),
                Ok(18) => msg.text = Some(r.read_string(bytes)?.to_owned()),
                Ok(tag) => { r.read_unknown(bytes, tag)?; }
                Err(error) => return Err(error),
            }
        }
        Ok(msg)
    }
}

impl MessageWrite for StreamErrorStanza {
    fn get_size(&self) -> usize {
        0
        + 1 + sizeof_len((&self.type_pb).len())
        + self.text.as_ref().map_or(0, |value| 1 + sizeof_len((value).len()))
    }

    fn write_message<W>(&self, writer: &mut Writer<W>) -> Result<()>
    where
        W: WriterBackend,
    {
        writer.write_with_tag(10, |writer| writer.write_string(&**&self.type_pb))?;
        if let Some(ref value) = self.text { writer.write_with_tag(18, |writer| writer.write_string(&**value))?; }
        Ok(())
    }

    fn write_file<P>(&self, p: P) -> Result<()>
    where
        P: AsRef<Path>,
    {
        let file = BufWriter::new(File::create(p)?);
        let mut writer = Writer::new(file);
        self.write_message(&mut writer)
    }
}

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct Close;

impl MessageRead<'_> for Close {
    fn from_reader(r: &mut BytesReader, _: &[u8]) -> Result<Self> {
        r.read_to_end();
        Ok(Self::default())
    }
}

impl MessageWrite for Close { 
    fn write_message<W>(&self, _: &mut Writer<W>) -> Result<()>
    where
        W: WriterBackend,
    {
        Ok(())
    }

    fn get_size(&self) -> usize {
        0
    }


    fn write_file<P>(&self, p: P) -> Result<()>
    where
        P: AsRef<Path>,
    {
        let file = BufWriter::new(File::create(p)?);
        let mut writer = Writer::new(file);
        self.write_message(&mut writer)
    }
}

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct Extension {
    pub id: i32,
    pub data: Vec<u8>,
}

impl<'a> MessageRead<'a> for Extension {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.id = r.read_int32(bytes)?,
                Ok(18) => r.read_bytes(bytes)?.clone_into(&mut msg.data),
                Ok(tag) => { r.read_unknown(bytes, tag)?; }
                Err(error) => return Err(error),
            }
        }
        Ok(msg)
    }
}

impl MessageWrite for Extension {
    fn get_size(&self) -> usize {
        0
        + 1 + sizeof_varint(*(&self.id) as u64)
        + 1 + sizeof_len((&self.data).len())
    }

    fn write_message<W>(&self, writer: &mut Writer<W>) -> Result<()>
    where
        W: WriterBackend,
    {
        writer.write_with_tag(8, |writer| writer.write_int32(*&self.id))?;
        writer.write_with_tag(18, |writer| writer.write_bytes(&**&self.data))?;
        Ok(())
    }

    fn write_file<P>(&self, p: P) -> Result<()>
    where
        P: AsRef<Path>,
    {
        let file = BufWriter::new(File::create(p)?);
        let mut writer = Writer::new(file);
        self.write_message(&mut writer)
    }
}

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct IqStanza {
    pub rmq_id: Option<i64>,
    pub type_pb: mod_IqStanza::IqType,
    pub id: String,
    pub from: Option<String>,
    pub to: Option<String>,
    pub error: Option<ErrorInfo>,
    pub extension: Option<Extension>,
    pub persistent_id: Option<String>,
    pub stream_id: Option<i32>,
    pub last_stream_id_received: Option<i32>,
    pub account_id: Option<i64>,
    pub status: Option<i64>,
}

impl<'a> MessageRead<'a> for IqStanza {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.rmq_id = Some(r.read_int64(bytes)?),
                Ok(16) => msg.type_pb = r.read_enum(bytes)?,
                Ok(26) => r.read_string(bytes)?.clone_into(&mut msg.id),
                Ok(34) => msg.from = Some(r.read_string(bytes)?.to_owned()),
                Ok(42) => msg.to = Some(r.read_string(bytes)?.to_owned()),
                Ok(50) => msg.error = Some(r.read_message::<ErrorInfo>(bytes)?),
                Ok(58) => msg.extension = Some(r.read_message::<Extension>(bytes)?),
                Ok(66) => msg.persistent_id = Some(r.read_string(bytes)?.to_owned()),
                Ok(72) => msg.stream_id = Some(r.read_int32(bytes)?),
                Ok(80) => msg.last_stream_id_received = Some(r.read_int32(bytes)?),
                Ok(88) => msg.account_id = Some(r.read_int64(bytes)?),
                Ok(96) => msg.status = Some(r.read_int64(bytes)?),
                Ok(tag) => { r.read_unknown(bytes, tag)?; }
                Err(error) => return Err(error),
            }
        }
        Ok(msg)
    }
}

impl MessageWrite for IqStanza {
    fn get_size(&self) -> usize {
        0
        + self.rmq_id.as_ref().map_or(0, |value| 1 + sizeof_varint(*(value) as u64))
        + 1 + sizeof_varint(*(&self.type_pb) as u64)
        + 1 + sizeof_len((&self.id).len())
        + self.from.as_ref().map_or(0, |value| 1 + sizeof_len((value).len()))
        + self.to.as_ref().map_or(0, |value| 1 + sizeof_len((value).len()))
        + self.error.as_ref().map_or(0, |value| 1 + sizeof_len((value).get_size()))
        + self.extension.as_ref().map_or(0, |value| 1 + sizeof_len((value).get_size()))
        + self.persistent_id.as_ref().map_or(0, |value| 1 + sizeof_len((value).len()))
        + self.stream_id.as_ref().map_or(0, |value| 1 + sizeof_varint(*(value) as u64))
        + self.last_stream_id_received.as_ref().map_or(0, |value| 1 + sizeof_varint(*(value) as u64))
        + self.account_id.as_ref().map_or(0, |value| 1 + sizeof_varint(*(value) as u64))
        + self.status.as_ref().map_or(0, |value| 1 + sizeof_varint(*(value) as u64))
    }

    fn write_message<W>(&self, writer: &mut Writer<W>) -> Result<()>
    where
        W: WriterBackend,
    {
        if let Some(ref value) = self.rmq_id { writer.write_with_tag(8, |writer| writer.write_int64(*value))?; }
        writer.write_with_tag(16, |writer| writer.write_enum(*&self.type_pb as i32))?;
        writer.write_with_tag(26, |writer| writer.write_string(&**&self.id))?;
        if let Some(ref value) = self.from { writer.write_with_tag(34, |writer| writer.write_string(&**value))?; }
        if let Some(ref value) = self.to { writer.write_with_tag(42, |writer| writer.write_string(&**value))?; }
        if let Some(ref value) = self.error { writer.write_with_tag(50, |writer| writer.write_message(value))?; }
        if let Some(ref value) = self.extension { writer.write_with_tag(58, |writer| writer.write_message(value))?; }
        if let Some(ref value) = self.persistent_id { writer.write_with_tag(66, |writer| writer.write_string(&**value))?; }
        if let Some(ref value) = self.stream_id { writer.write_with_tag(72, |writer| writer.write_int32(*value))?; }
        if let Some(ref value) = self.last_stream_id_received { writer.write_with_tag(80, |writer| writer.write_int32(*value))?; }
        if let Some(ref value) = self.account_id { writer.write_with_tag(88, |writer| writer.write_int64(*value))?; }
        if let Some(ref value) = self.status { writer.write_with_tag(96, |writer| writer.write_int64(*value))?; }
        Ok(())
    }

    fn write_file<P>(&self, p: P) -> Result<()>
    where
        P: AsRef<Path>,
    {
        let file = BufWriter::new(File::create(p)?);
        let mut writer = Writer::new(file);
        self.write_message(&mut writer)
    }
}

pub mod mod_IqStanza {


#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum IqType {
    GET = 0,
    SET = 1,
    RESULT = 2,
    IQ_ERROR = 3,
}

impl Default for IqType {
    fn default() -> Self {
        Self::GET
    }
}

impl From<i32> for IqType {
    fn from(value: i32) -> Self {
        match value {
            0 => Self::GET,
            1 => Self::SET,
            2 => Self::RESULT,
            3 => Self::IQ_ERROR,
            _ => Self::default(),
        }
    }
}

impl<'a> From<&'a str> for IqType {
    fn from(value: &'a str) -> Self {
        match value {
            "GET" => Self::GET,
            "SET" => Self::SET,
            "RESULT" => Self::RESULT,
            "IQ_ERROR" => Self::IQ_ERROR,
            _ => Self::default(),
        }
    }
}

}

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct AppData {
    pub key: String,
    pub value: String,
}

impl<'a> MessageRead<'a> for AppData {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => r.read_string(bytes)?.clone_into(&mut msg.key),
                Ok(18) => r.read_string(bytes)?.clone_into(&mut msg.value),
                Ok(tag) => { r.read_unknown(bytes, tag)?; }
                Err(error) => return Err(error),
            }
        }
        Ok(msg)
    }
}

impl MessageWrite for AppData {
    fn get_size(&self) -> usize {
        0
        + 1 + sizeof_len((&self.key).len())
        + 1 + sizeof_len((&self.value).len())
    }

    fn write_message<W>(&self, writer: &mut Writer<W>) -> Result<()>
    where
        W: WriterBackend,
    {
        writer.write_with_tag(10, |writer| writer.write_string(&**&self.key))?;
        writer.write_with_tag(18, |writer| writer.write_string(&**&self.value))?;
        Ok(())
    }

    fn write_file<P>(&self, p: P) -> Result<()>
    where
        P: AsRef<Path>,
    {
        let file = BufWriter::new(File::create(p)?);
        let mut writer = Writer::new(file);
        self.write_message(&mut writer)
    }
}

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct DataMessageStanza {
    pub id: Option<String>,
    pub from: String,
    pub to: Option<String>,
    pub category: String,
    pub token: Option<String>,
    pub app_data: Vec<AppData>,
    pub from_trusted_server: Option<bool>,
    pub persistent_id: Option<String>,
    pub stream_id: Option<i32>,
    pub last_stream_id_received: Option<i32>,
    pub reg_id: Option<String>,
    pub device_user_id: Option<i64>,
    pub ttl: Option<i32>,
    pub sent: Option<i64>,
    pub queued: Option<i32>,
    pub status: Option<i64>,
    pub raw_data: Option<Vec<u8>>,
    pub immediate_ack: Option<bool>,
}

impl<'a> MessageRead<'a> for DataMessageStanza {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(18) => msg.id = Some(r.read_string(bytes)?.to_owned()),
                Ok(26) => r.read_string(bytes)?.clone_into(&mut msg.from),
                Ok(34) => msg.to = Some(r.read_string(bytes)?.to_owned()),
                Ok(42) => r.read_string(bytes)?.clone_into(&mut msg.category),
                Ok(50) => msg.token = Some(r.read_string(bytes)?.to_owned()),
                Ok(58) => msg.app_data.push(r.read_message::<AppData>(bytes)?),
                Ok(64) => msg.from_trusted_server = Some(r.read_bool(bytes)?),
                Ok(74) => msg.persistent_id = Some(r.read_string(bytes)?.to_owned()),
                Ok(80) => msg.stream_id = Some(r.read_int32(bytes)?),
                Ok(88) => msg.last_stream_id_received = Some(r.read_int32(bytes)?),
                Ok(106) => msg.reg_id = Some(r.read_string(bytes)?.to_owned()),
                Ok(128) => msg.device_user_id = Some(r.read_int64(bytes)?),
                Ok(136) => msg.ttl = Some(r.read_int32(bytes)?),
                Ok(144) => msg.sent = Some(r.read_int64(bytes)?),
                Ok(152) => msg.queued = Some(r.read_int32(bytes)?),
                Ok(160) => msg.status = Some(r.read_int64(bytes)?),
                Ok(170) => msg.raw_data = Some(r.read_bytes(bytes)?.to_owned()),
                Ok(192) => msg.immediate_ack = Some(r.read_bool(bytes)?),
                Ok(tag) => { r.read_unknown(bytes, tag)?; }
                Err(error) => return Err(error),
            }
        }
        Ok(msg)
    }
}

impl MessageWrite for DataMessageStanza {
    fn get_size(&self) -> usize {
        0
        + self.id.as_ref().map_or(0, |value| 1 + sizeof_len((value).len()))
        + 1 + sizeof_len((&self.from).len())
        + self.to.as_ref().map_or(0, |value| 1 + sizeof_len((value).len()))
        + 1 + sizeof_len((&self.category).len())
        + self.token.as_ref().map_or(0, |value| 1 + sizeof_len((value).len()))
        + self.app_data.iter().map(|value| 1 + sizeof_len((value).get_size())).sum::<usize>()
        + self.from_trusted_server.as_ref().map_or(0, |value| 1 + sizeof_varint(u64::from(*value)))
        + self.persistent_id.as_ref().map_or(0, |value| 1 + sizeof_len((value).len()))
        + self.stream_id.as_ref().map_or(0, |value| 1 + sizeof_varint(*(value) as u64))
        + self.last_stream_id_received.as_ref().map_or(0, |value| 1 + sizeof_varint(*(value) as u64))
        + self.reg_id.as_ref().map_or(0, |value| 1 + sizeof_len((value).len()))
        + self.device_user_id.as_ref().map_or(0, |value| 2 + sizeof_varint(*(value) as u64))
        + self.ttl.as_ref().map_or(0, |value| 2 + sizeof_varint(*(value) as u64))
        + self.sent.as_ref().map_or(0, |value| 2 + sizeof_varint(*(value) as u64))
        + self.queued.as_ref().map_or(0, |value| 2 + sizeof_varint(*(value) as u64))
        + self.status.as_ref().map_or(0, |value| 2 + sizeof_varint(*(value) as u64))
        + self.raw_data.as_ref().map_or(0, |value| 2 + sizeof_len((value).len()))
        + self.immediate_ack.as_ref().map_or(0, |value| 2 + sizeof_varint(u64::from(*value)))
    }

    fn write_message<W>(&self, writer: &mut Writer<W>) -> Result<()>
    where
        W: WriterBackend,
    {
        if let Some(ref value) = self.id { writer.write_with_tag(18, |writer| writer.write_string(&**value))?; }
        writer.write_with_tag(26, |writer| writer.write_string(&**&self.from))?;
        if let Some(ref value) = self.to { writer.write_with_tag(34, |writer| writer.write_string(&**value))?; }
        writer.write_with_tag(42, |writer| writer.write_string(&**&self.category))?;
        if let Some(ref value) = self.token { writer.write_with_tag(50, |writer| writer.write_string(&**value))?; }
        for value in &self.app_data { writer.write_with_tag(58, |writer| writer.write_message(value))?; }
        if let Some(ref value) = self.from_trusted_server { writer.write_with_tag(64, |writer| writer.write_bool(*value))?; }
        if let Some(ref value) = self.persistent_id { writer.write_with_tag(74, |writer| writer.write_string(&**value))?; }
        if let Some(ref value) = self.stream_id { writer.write_with_tag(80, |writer| writer.write_int32(*value))?; }
        if let Some(ref value) = self.last_stream_id_received { writer.write_with_tag(88, |writer| writer.write_int32(*value))?; }
        if let Some(ref value) = self.reg_id { writer.write_with_tag(106, |writer| writer.write_string(&**value))?; }
        if let Some(ref value) = self.device_user_id { writer.write_with_tag(128, |writer| writer.write_int64(*value))?; }
        if let Some(ref value) = self.ttl { writer.write_with_tag(136, |writer| writer.write_int32(*value))?; }
        if let Some(ref value) = self.sent { writer.write_with_tag(144, |writer| writer.write_int64(*value))?; }
        if let Some(ref value) = self.queued { writer.write_with_tag(152, |writer| writer.write_int32(*value))?; }
        if let Some(ref value) = self.status { writer.write_with_tag(160, |writer| writer.write_int64(*value))?; }
        if let Some(ref value) = self.raw_data { writer.write_with_tag(170, |writer| writer.write_bytes(&**value))?; }
        if let Some(ref value) = self.immediate_ack { writer.write_with_tag(192, |writer| writer.write_bool(*value))?; }
        Ok(())
    }

    fn write_file<P>(&self, p: P) -> Result<()>
    where
        P: AsRef<Path>,
    {
        let file = BufWriter::new(File::create(p)?);
        let mut writer = Writer::new(file);
        self.write_message(&mut writer)
    }
}

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct StreamAck;

impl MessageRead<'_> for StreamAck {
    fn from_reader(r: &mut BytesReader, _: &[u8]) -> Result<Self> {
        r.read_to_end();
        Ok(Self::default())
    }
}

impl MessageWrite for StreamAck { 
    fn write_message<W>(&self, _: &mut Writer<W>) -> Result<()>
    where
        W: WriterBackend,
    {
        Ok(())
    }

    fn get_size(&self) -> usize {
        0
    }


    fn write_file<P>(&self, p: P) -> Result<()>
    where
        P: AsRef<Path>,
    {
        let file = BufWriter::new(File::create(p)?);
        let mut writer = Writer::new(file);
        self.write_message(&mut writer)
    }
}

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct SelectiveAck {
    pub id: Vec<String>,
}

impl<'a> MessageRead<'a> for SelectiveAck {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.id.push(r.read_string(bytes)?.to_owned()),
                Ok(tag) => { r.read_unknown(bytes, tag)?; }
                Err(error) => return Err(error),
            }
        }
        Ok(msg)
    }
}

impl MessageWrite for SelectiveAck {
    fn get_size(&self) -> usize {
        0
        + self.id.iter().map(|value| 1 + sizeof_len((value).len())).sum::<usize>()
    }

    fn write_message<W>(&self, writer: &mut Writer<W>) -> Result<()>
    where
        W: WriterBackend,
    {
        for value in &self.id { writer.write_with_tag(10, |writer| writer.write_string(&**value))?; }
        Ok(())
    }

    fn write_file<P>(&self, p: P) -> Result<()>
    where
        P: AsRef<Path>,
    {
        let file = BufWriter::new(File::create(p)?);
        let mut writer = Writer::new(file);
        self.write_message(&mut writer)
    }
}

