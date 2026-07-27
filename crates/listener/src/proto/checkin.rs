// Automatically generated rust module for 'checkin.proto' file
// Regenerate from the repository root: nix develop -c nu nix/regen-proto.nu

#![allow(non_snake_case, reason = "generated protobuf code")]
#![allow(non_upper_case_globals, reason = "generated protobuf code")]
#![allow(non_camel_case_types, reason = "generated protobuf code")]
#![allow(unused_imports, reason = "generated protobuf code")]
#![allow(unknown_lints, reason = "generated protobuf code")]
#![allow(clippy::all, reason = "generated protobuf code")]
#![cfg_attr(rustfmt, rustfmt_skip)]

use std::{fs::File, io::BufWriter, path::Path};


use quick_protobuf::{MessageInfo, MessageRead, MessageWrite, BytesReader, Writer, WriterBackend, Result};
use quick_protobuf::sizeofs::{sizeof_len, sizeof_varint};
use super::*;

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct GservicesSetting {
    pub name: Vec<u8>,
    pub value: Vec<u8>,
}

impl<'a> MessageRead<'a> for GservicesSetting {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => r.read_bytes(bytes)?.clone_into(&mut msg.name),
                Ok(18) => r.read_bytes(bytes)?.clone_into(&mut msg.value),
                Ok(tag) => { r.read_unknown(bytes, tag)?; }
                Err(error) => return Err(error),
            }
        }
        Ok(msg)
    }
}

impl MessageWrite for GservicesSetting {
    fn get_size(&self) -> usize {
        0
        + 1 + sizeof_len((&self.name).len())
        + 1 + sizeof_len((&self.value).len())
    }

    fn write_message<W>(&self, writer: &mut Writer<W>) -> Result<()>
    where
        W: WriterBackend,
    {
        writer.write_with_tag(10, |writer| writer.write_bytes(&**&self.name))?;
        writer.write_with_tag(18, |writer| writer.write_bytes(&**&self.value))?;
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
pub struct AndroidCheckinRequest {
    pub imei: Option<String>,
    pub meid: Option<String>,
    pub mac_addr: Vec<String>,
    pub mac_addr_type: Vec<String>,
    pub serial_number: Option<String>,
    pub esn: Option<String>,
    pub id: Option<i64>,
    pub logging_id: Option<i64>,
    pub digest: Option<String>,
    pub locale: Option<String>,
    pub checkin: super::android_checkin::AndroidCheckinProto,
    pub desired_build: Option<String>,
    pub market_checkin: Option<String>,
    pub account_cookie: Vec<String>,
    pub time_zone: Option<String>,
    pub security_token: Option<u64>,
    pub version: Option<i32>,
    pub ota_cert: Vec<String>,
    pub fragment: Option<i32>,
    pub user_name: Option<String>,
    pub user_serial_number: Option<i32>,
}

impl<'a> MessageRead<'a> for AndroidCheckinRequest {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.imei = Some(r.read_string(bytes)?.to_owned()),
                Ok(82) => msg.meid = Some(r.read_string(bytes)?.to_owned()),
                Ok(74) => msg.mac_addr.push(r.read_string(bytes)?.to_owned()),
                Ok(154) => msg.mac_addr_type.push(r.read_string(bytes)?.to_owned()),
                Ok(130) => msg.serial_number = Some(r.read_string(bytes)?.to_owned()),
                Ok(138) => msg.esn = Some(r.read_string(bytes)?.to_owned()),
                Ok(16) => msg.id = Some(r.read_int64(bytes)?),
                Ok(56) => msg.logging_id = Some(r.read_int64(bytes)?),
                Ok(26) => msg.digest = Some(r.read_string(bytes)?.to_owned()),
                Ok(50) => msg.locale = Some(r.read_string(bytes)?.to_owned()),
                Ok(34) => msg.checkin = r.read_message::<super::android_checkin::AndroidCheckinProto>(bytes)?,
                Ok(42) => msg.desired_build = Some(r.read_string(bytes)?.to_owned()),
                Ok(66) => msg.market_checkin = Some(r.read_string(bytes)?.to_owned()),
                Ok(90) => msg.account_cookie.push(r.read_string(bytes)?.to_owned()),
                Ok(98) => msg.time_zone = Some(r.read_string(bytes)?.to_owned()),
                Ok(105) => msg.security_token = Some(r.read_fixed64(bytes)?),
                Ok(112) => msg.version = Some(r.read_int32(bytes)?),
                Ok(122) => msg.ota_cert.push(r.read_string(bytes)?.to_owned()),
                Ok(160) => msg.fragment = Some(r.read_int32(bytes)?),
                Ok(170) => msg.user_name = Some(r.read_string(bytes)?.to_owned()),
                Ok(176) => msg.user_serial_number = Some(r.read_int32(bytes)?),
                Ok(tag) => { r.read_unknown(bytes, tag)?; }
                Err(error) => return Err(error),
            }
        }
        Ok(msg)
    }
}

impl MessageWrite for AndroidCheckinRequest {
    fn get_size(&self) -> usize {
        0
        + self.imei.as_ref().map_or(0, |value| 1 + sizeof_len((value).len()))
        + self.meid.as_ref().map_or(0, |value| 1 + sizeof_len((value).len()))
        + self.mac_addr.iter().map(|value| 1 + sizeof_len((value).len())).sum::<usize>()
        + self.mac_addr_type.iter().map(|value| 2 + sizeof_len((value).len())).sum::<usize>()
        + self.serial_number.as_ref().map_or(0, |value| 2 + sizeof_len((value).len()))
        + self.esn.as_ref().map_or(0, |value| 2 + sizeof_len((value).len()))
        + self.id.as_ref().map_or(0, |value| 1 + sizeof_varint(*(value) as u64))
        + self.logging_id.as_ref().map_or(0, |value| 1 + sizeof_varint(*(value) as u64))
        + self.digest.as_ref().map_or(0, |value| 1 + sizeof_len((value).len()))
        + self.locale.as_ref().map_or(0, |value| 1 + sizeof_len((value).len()))
        + 1 + sizeof_len((&self.checkin).get_size())
        + self.desired_build.as_ref().map_or(0, |value| 1 + sizeof_len((value).len()))
        + self.market_checkin.as_ref().map_or(0, |value| 1 + sizeof_len((value).len()))
        + self.account_cookie.iter().map(|value| 1 + sizeof_len((value).len())).sum::<usize>()
        + self.time_zone.as_ref().map_or(0, |value| 1 + sizeof_len((value).len()))
        + self.security_token.as_ref().map_or(0, |_| 1 + 8)
        + self.version.as_ref().map_or(0, |value| 1 + sizeof_varint(*(value) as u64))
        + self.ota_cert.iter().map(|value| 1 + sizeof_len((value).len())).sum::<usize>()
        + self.fragment.as_ref().map_or(0, |value| 2 + sizeof_varint(*(value) as u64))
        + self.user_name.as_ref().map_or(0, |value| 2 + sizeof_len((value).len()))
        + self.user_serial_number.as_ref().map_or(0, |value| 2 + sizeof_varint(*(value) as u64))
    }

    fn write_message<W>(&self, writer: &mut Writer<W>) -> Result<()>
    where
        W: WriterBackend,
    {
        if let Some(ref value) = self.imei { writer.write_with_tag(10, |writer| writer.write_string(&**value))?; }
        if let Some(ref value) = self.meid { writer.write_with_tag(82, |writer| writer.write_string(&**value))?; }
        for value in &self.mac_addr { writer.write_with_tag(74, |writer| writer.write_string(&**value))?; }
        for value in &self.mac_addr_type { writer.write_with_tag(154, |writer| writer.write_string(&**value))?; }
        if let Some(ref value) = self.serial_number { writer.write_with_tag(130, |writer| writer.write_string(&**value))?; }
        if let Some(ref value) = self.esn { writer.write_with_tag(138, |writer| writer.write_string(&**value))?; }
        if let Some(ref value) = self.id { writer.write_with_tag(16, |writer| writer.write_int64(*value))?; }
        if let Some(ref value) = self.logging_id { writer.write_with_tag(56, |writer| writer.write_int64(*value))?; }
        if let Some(ref value) = self.digest { writer.write_with_tag(26, |writer| writer.write_string(&**value))?; }
        if let Some(ref value) = self.locale { writer.write_with_tag(50, |writer| writer.write_string(&**value))?; }
        writer.write_with_tag(34, |writer| writer.write_message(&self.checkin))?;
        if let Some(ref value) = self.desired_build { writer.write_with_tag(42, |writer| writer.write_string(&**value))?; }
        if let Some(ref value) = self.market_checkin { writer.write_with_tag(66, |writer| writer.write_string(&**value))?; }
        for value in &self.account_cookie { writer.write_with_tag(90, |writer| writer.write_string(&**value))?; }
        if let Some(ref value) = self.time_zone { writer.write_with_tag(98, |writer| writer.write_string(&**value))?; }
        if let Some(ref value) = self.security_token { writer.write_with_tag(105, |writer| writer.write_fixed64(*value))?; }
        if let Some(ref value) = self.version { writer.write_with_tag(112, |writer| writer.write_int32(*value))?; }
        for value in &self.ota_cert { writer.write_with_tag(122, |writer| writer.write_string(&**value))?; }
        if let Some(ref value) = self.fragment { writer.write_with_tag(160, |writer| writer.write_int32(*value))?; }
        if let Some(ref value) = self.user_name { writer.write_with_tag(170, |writer| writer.write_string(&**value))?; }
        if let Some(ref value) = self.user_serial_number { writer.write_with_tag(176, |writer| writer.write_int32(*value))?; }
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
pub struct AndroidCheckinResponse {
    pub stats_ok: bool,
    pub time_msec: Option<i64>,
    pub digest: Option<String>,
    pub settings_diff: Option<bool>,
    pub delete_setting: Vec<String>,
    pub setting: Vec<GservicesSetting>,
    pub market_ok: Option<bool>,
    pub android_id: Option<u64>,
    pub security_token: Option<u64>,
    pub version_info: Option<String>,
}

impl<'a> MessageRead<'a> for AndroidCheckinResponse {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.stats_ok = r.read_bool(bytes)?,
                Ok(24) => msg.time_msec = Some(r.read_int64(bytes)?),
                Ok(34) => msg.digest = Some(r.read_string(bytes)?.to_owned()),
                Ok(72) => msg.settings_diff = Some(r.read_bool(bytes)?),
                Ok(82) => msg.delete_setting.push(r.read_string(bytes)?.to_owned()),
                Ok(42) => msg.setting.push(r.read_message::<GservicesSetting>(bytes)?),
                Ok(48) => msg.market_ok = Some(r.read_bool(bytes)?),
                Ok(57) => msg.android_id = Some(r.read_fixed64(bytes)?),
                Ok(65) => msg.security_token = Some(r.read_fixed64(bytes)?),
                Ok(90) => msg.version_info = Some(r.read_string(bytes)?.to_owned()),
                Ok(tag) => { r.read_unknown(bytes, tag)?; }
                Err(error) => return Err(error),
            }
        }
        Ok(msg)
    }
}

impl MessageWrite for AndroidCheckinResponse {
    fn get_size(&self) -> usize {
        0
        + 1 + sizeof_varint(u64::from(self.stats_ok))
        + self.time_msec.as_ref().map_or(0, |value| 1 + sizeof_varint(*(value) as u64))
        + self.digest.as_ref().map_or(0, |value| 1 + sizeof_len((value).len()))
        + self.settings_diff.as_ref().map_or(0, |value| 1 + sizeof_varint(u64::from(*value)))
        + self.delete_setting.iter().map(|value| 1 + sizeof_len((value).len())).sum::<usize>()
        + self.setting.iter().map(|value| 1 + sizeof_len((value).get_size())).sum::<usize>()
        + self.market_ok.as_ref().map_or(0, |value| 1 + sizeof_varint(u64::from(*value)))
        + self.android_id.as_ref().map_or(0, |_| 1 + 8)
        + self.security_token.as_ref().map_or(0, |_| 1 + 8)
        + self.version_info.as_ref().map_or(0, |value| 1 + sizeof_len((value).len()))
    }

    fn write_message<W>(&self, writer: &mut Writer<W>) -> Result<()>
    where
        W: WriterBackend,
    {
        writer.write_with_tag(8, |writer| writer.write_bool(*&self.stats_ok))?;
        if let Some(ref value) = self.time_msec { writer.write_with_tag(24, |writer| writer.write_int64(*value))?; }
        if let Some(ref value) = self.digest { writer.write_with_tag(34, |writer| writer.write_string(&**value))?; }
        if let Some(ref value) = self.settings_diff { writer.write_with_tag(72, |writer| writer.write_bool(*value))?; }
        for value in &self.delete_setting { writer.write_with_tag(82, |writer| writer.write_string(&**value))?; }
        for value in &self.setting { writer.write_with_tag(42, |writer| writer.write_message(value))?; }
        if let Some(ref value) = self.market_ok { writer.write_with_tag(48, |writer| writer.write_bool(*value))?; }
        if let Some(ref value) = self.android_id { writer.write_with_tag(57, |writer| writer.write_fixed64(*value))?; }
        if let Some(ref value) = self.security_token { writer.write_with_tag(65, |writer| writer.write_fixed64(*value))?; }
        if let Some(ref value) = self.version_info { writer.write_with_tag(90, |writer| writer.write_string(&**value))?; }
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

