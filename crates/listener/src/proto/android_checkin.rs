// Automatically generated rust module for 'android_checkin.proto' file
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

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum DeviceType {
    DEVICE_ANDROID_OS = 1,
    DEVICE_IOS_OS = 2,
    DEVICE_CHROME_BROWSER = 3,
    DEVICE_CHROME_OS = 4,
}

impl Default for DeviceType {
    fn default() -> Self {
        Self::DEVICE_ANDROID_OS
    }
}

impl From<i32> for DeviceType {
    fn from(value: i32) -> Self {
        match value {
            1 => Self::DEVICE_ANDROID_OS,
            2 => Self::DEVICE_IOS_OS,
            3 => Self::DEVICE_CHROME_BROWSER,
            4 => Self::DEVICE_CHROME_OS,
            _ => Self::default(),
        }
    }
}

impl<'a> From<&'a str> for DeviceType {
    fn from(value: &'a str) -> Self {
        match value {
            "DEVICE_ANDROID_OS" => Self::DEVICE_ANDROID_OS,
            "DEVICE_IOS_OS" => Self::DEVICE_IOS_OS,
            "DEVICE_CHROME_BROWSER" => Self::DEVICE_CHROME_BROWSER,
            "DEVICE_CHROME_OS" => Self::DEVICE_CHROME_OS,
            _ => Self::default(),
        }
    }
}

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct ChromeBuildProto {
    pub platform: Option<mod_ChromeBuildProto::Platform>,
    pub chrome_version: Option<String>,
    pub channel: Option<mod_ChromeBuildProto::Channel>,
}

impl<'a> MessageRead<'a> for ChromeBuildProto {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.platform = Some(r.read_enum(bytes)?),
                Ok(18) => msg.chrome_version = Some(r.read_string(bytes)?.to_owned()),
                Ok(24) => msg.channel = Some(r.read_enum(bytes)?),
                Ok(tag) => { r.read_unknown(bytes, tag)?; }
                Err(error) => return Err(error),
            }
        }
        Ok(msg)
    }
}

impl MessageWrite for ChromeBuildProto {
    fn get_size(&self) -> usize {
        0
        + self.platform.as_ref().map_or(0, |value| 1 + sizeof_varint(*(value) as u64))
        + self.chrome_version.as_ref().map_or(0, |value| 1 + sizeof_len((value).len()))
        + self.channel.as_ref().map_or(0, |value| 1 + sizeof_varint(*(value) as u64))
    }

    fn write_message<W>(&self, writer: &mut Writer<W>) -> Result<()>
    where
        W: WriterBackend,
    {
        if let Some(ref value) = self.platform { writer.write_with_tag(8, |writer| writer.write_enum(*value as i32))?; }
        if let Some(ref value) = self.chrome_version { writer.write_with_tag(18, |writer| writer.write_string(&**value))?; }
        if let Some(ref value) = self.channel { writer.write_with_tag(24, |writer| writer.write_enum(*value as i32))?; }
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

pub mod mod_ChromeBuildProto {


#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Platform {
    PLATFORM_WIN = 1,
    PLATFORM_MAC = 2,
    PLATFORM_LINUX = 3,
    PLATFORM_CROS = 4,
    PLATFORM_IOS = 5,
    PLATFORM_ANDROID = 6,
}

impl Default for Platform {
    fn default() -> Self {
        Self::PLATFORM_WIN
    }
}

impl From<i32> for Platform {
    fn from(value: i32) -> Self {
        match value {
            1 => Self::PLATFORM_WIN,
            2 => Self::PLATFORM_MAC,
            3 => Self::PLATFORM_LINUX,
            4 => Self::PLATFORM_CROS,
            5 => Self::PLATFORM_IOS,
            6 => Self::PLATFORM_ANDROID,
            _ => Self::default(),
        }
    }
}

impl<'a> From<&'a str> for Platform {
    fn from(value: &'a str) -> Self {
        match value {
            "PLATFORM_WIN" => Self::PLATFORM_WIN,
            "PLATFORM_MAC" => Self::PLATFORM_MAC,
            "PLATFORM_LINUX" => Self::PLATFORM_LINUX,
            "PLATFORM_CROS" => Self::PLATFORM_CROS,
            "PLATFORM_IOS" => Self::PLATFORM_IOS,
            "PLATFORM_ANDROID" => Self::PLATFORM_ANDROID,
            _ => Self::default(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Channel {
    CHANNEL_STABLE = 1,
    CHANNEL_BETA = 2,
    CHANNEL_DEV = 3,
    CHANNEL_CANARY = 4,
    CHANNEL_UNKNOWN = 5,
}

impl Default for Channel {
    fn default() -> Self {
        Self::CHANNEL_STABLE
    }
}

impl From<i32> for Channel {
    fn from(value: i32) -> Self {
        match value {
            1 => Self::CHANNEL_STABLE,
            2 => Self::CHANNEL_BETA,
            3 => Self::CHANNEL_DEV,
            4 => Self::CHANNEL_CANARY,
            5 => Self::CHANNEL_UNKNOWN,
            _ => Self::default(),
        }
    }
}

impl<'a> From<&'a str> for Channel {
    fn from(value: &'a str) -> Self {
        match value {
            "CHANNEL_STABLE" => Self::CHANNEL_STABLE,
            "CHANNEL_BETA" => Self::CHANNEL_BETA,
            "CHANNEL_DEV" => Self::CHANNEL_DEV,
            "CHANNEL_CANARY" => Self::CHANNEL_CANARY,
            "CHANNEL_UNKNOWN" => Self::CHANNEL_UNKNOWN,
            _ => Self::default(),
        }
    }
}

}

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct AndroidBuildProto {
    pub fingerprint: Option<String>,
    pub hardware: Option<String>,
    pub brand: Option<String>,
    pub radio: Option<String>,
    pub bootloader: Option<String>,
    pub client_id: Option<String>,
    pub time: Option<i64>,
    pub package_version_code: Option<i32>,
    pub device: Option<String>,
    pub sdk_version: Option<i32>,
    pub model: Option<String>,
    pub manufacturer: Option<String>,
    pub product: Option<String>,
    pub ota_installed: Option<bool>,
}

impl<'a> MessageRead<'a> for AndroidBuildProto {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.fingerprint = Some(r.read_string(bytes)?.to_owned()),
                Ok(18) => msg.hardware = Some(r.read_string(bytes)?.to_owned()),
                Ok(26) => msg.brand = Some(r.read_string(bytes)?.to_owned()),
                Ok(34) => msg.radio = Some(r.read_string(bytes)?.to_owned()),
                Ok(42) => msg.bootloader = Some(r.read_string(bytes)?.to_owned()),
                Ok(50) => msg.client_id = Some(r.read_string(bytes)?.to_owned()),
                Ok(56) => msg.time = Some(r.read_int64(bytes)?),
                Ok(64) => msg.package_version_code = Some(r.read_int32(bytes)?),
                Ok(74) => msg.device = Some(r.read_string(bytes)?.to_owned()),
                Ok(80) => msg.sdk_version = Some(r.read_int32(bytes)?),
                Ok(90) => msg.model = Some(r.read_string(bytes)?.to_owned()),
                Ok(98) => msg.manufacturer = Some(r.read_string(bytes)?.to_owned()),
                Ok(106) => msg.product = Some(r.read_string(bytes)?.to_owned()),
                Ok(112) => msg.ota_installed = Some(r.read_bool(bytes)?),
                Ok(tag) => { r.read_unknown(bytes, tag)?; }
                Err(error) => return Err(error),
            }
        }
        Ok(msg)
    }
}

impl MessageWrite for AndroidBuildProto {
    fn get_size(&self) -> usize {
        0
        + self.fingerprint.as_ref().map_or(0, |value| 1 + sizeof_len((value).len()))
        + self.hardware.as_ref().map_or(0, |value| 1 + sizeof_len((value).len()))
        + self.brand.as_ref().map_or(0, |value| 1 + sizeof_len((value).len()))
        + self.radio.as_ref().map_or(0, |value| 1 + sizeof_len((value).len()))
        + self.bootloader.as_ref().map_or(0, |value| 1 + sizeof_len((value).len()))
        + self.client_id.as_ref().map_or(0, |value| 1 + sizeof_len((value).len()))
        + self.time.as_ref().map_or(0, |value| 1 + sizeof_varint(*(value) as u64))
        + self.package_version_code.as_ref().map_or(0, |value| 1 + sizeof_varint(*(value) as u64))
        + self.device.as_ref().map_or(0, |value| 1 + sizeof_len((value).len()))
        + self.sdk_version.as_ref().map_or(0, |value| 1 + sizeof_varint(*(value) as u64))
        + self.model.as_ref().map_or(0, |value| 1 + sizeof_len((value).len()))
        + self.manufacturer.as_ref().map_or(0, |value| 1 + sizeof_len((value).len()))
        + self.product.as_ref().map_or(0, |value| 1 + sizeof_len((value).len()))
        + self.ota_installed.as_ref().map_or(0, |value| 1 + sizeof_varint(u64::from(*value)))
    }

    fn write_message<W>(&self, writer: &mut Writer<W>) -> Result<()>
    where
        W: WriterBackend,
    {
        if let Some(ref value) = self.fingerprint { writer.write_with_tag(10, |writer| writer.write_string(&**value))?; }
        if let Some(ref value) = self.hardware { writer.write_with_tag(18, |writer| writer.write_string(&**value))?; }
        if let Some(ref value) = self.brand { writer.write_with_tag(26, |writer| writer.write_string(&**value))?; }
        if let Some(ref value) = self.radio { writer.write_with_tag(34, |writer| writer.write_string(&**value))?; }
        if let Some(ref value) = self.bootloader { writer.write_with_tag(42, |writer| writer.write_string(&**value))?; }
        if let Some(ref value) = self.client_id { writer.write_with_tag(50, |writer| writer.write_string(&**value))?; }
        if let Some(ref value) = self.time { writer.write_with_tag(56, |writer| writer.write_int64(*value))?; }
        if let Some(ref value) = self.package_version_code { writer.write_with_tag(64, |writer| writer.write_int32(*value))?; }
        if let Some(ref value) = self.device { writer.write_with_tag(74, |writer| writer.write_string(&**value))?; }
        if let Some(ref value) = self.sdk_version { writer.write_with_tag(80, |writer| writer.write_int32(*value))?; }
        if let Some(ref value) = self.model { writer.write_with_tag(90, |writer| writer.write_string(&**value))?; }
        if let Some(ref value) = self.manufacturer { writer.write_with_tag(98, |writer| writer.write_string(&**value))?; }
        if let Some(ref value) = self.product { writer.write_with_tag(106, |writer| writer.write_string(&**value))?; }
        if let Some(ref value) = self.ota_installed { writer.write_with_tag(112, |writer| writer.write_bool(*value))?; }
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
pub struct AndroidCheckinEvent {
    pub tag: Option<String>,
    pub value: Option<String>,
    pub time_msec: Option<i64>,
}

impl<'a> MessageRead<'a> for AndroidCheckinEvent {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.tag = Some(r.read_string(bytes)?.to_owned()),
                Ok(18) => msg.value = Some(r.read_string(bytes)?.to_owned()),
                Ok(24) => msg.time_msec = Some(r.read_int64(bytes)?),
                Ok(tag) => { r.read_unknown(bytes, tag)?; }
                Err(error) => return Err(error),
            }
        }
        Ok(msg)
    }
}

impl MessageWrite for AndroidCheckinEvent {
    fn get_size(&self) -> usize {
        0
        + self.tag.as_ref().map_or(0, |value| 1 + sizeof_len((value).len()))
        + self.value.as_ref().map_or(0, |value| 1 + sizeof_len((value).len()))
        + self.time_msec.as_ref().map_or(0, |value| 1 + sizeof_varint(*(value) as u64))
    }

    fn write_message<W>(&self, writer: &mut Writer<W>) -> Result<()>
    where
        W: WriterBackend,
    {
        if let Some(ref value) = self.tag { writer.write_with_tag(10, |writer| writer.write_string(&**value))?; }
        if let Some(ref value) = self.value { writer.write_with_tag(18, |writer| writer.write_string(&**value))?; }
        if let Some(ref value) = self.time_msec { writer.write_with_tag(24, |writer| writer.write_int64(*value))?; }
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
pub struct AndroidCheckinProto {
    pub build: Option<AndroidBuildProto>,
    pub last_checkin_msec: Option<i64>,
    pub event: Vec<AndroidCheckinEvent>,
    pub cell_operator: Option<String>,
    pub sim_operator: Option<String>,
    pub roaming: Option<String>,
    pub user_number: Option<i32>,
    pub type_pb: DeviceType,
    pub chrome_build: Option<ChromeBuildProto>,
}

impl<'a> MessageRead<'a> for AndroidCheckinProto {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.build = Some(r.read_message::<AndroidBuildProto>(bytes)?),
                Ok(16) => msg.last_checkin_msec = Some(r.read_int64(bytes)?),
                Ok(26) => msg.event.push(r.read_message::<AndroidCheckinEvent>(bytes)?),
                Ok(50) => msg.cell_operator = Some(r.read_string(bytes)?.to_owned()),
                Ok(58) => msg.sim_operator = Some(r.read_string(bytes)?.to_owned()),
                Ok(66) => msg.roaming = Some(r.read_string(bytes)?.to_owned()),
                Ok(72) => msg.user_number = Some(r.read_int32(bytes)?),
                Ok(96) => msg.type_pb = r.read_enum(bytes)?,
                Ok(106) => msg.chrome_build = Some(r.read_message::<ChromeBuildProto>(bytes)?),
                Ok(tag) => { r.read_unknown(bytes, tag)?; }
                Err(error) => return Err(error),
            }
        }
        Ok(msg)
    }
}

impl MessageWrite for AndroidCheckinProto {
    fn get_size(&self) -> usize {
        0
        + self.build.as_ref().map_or(0, |value| 1 + sizeof_len((value).get_size()))
        + self.last_checkin_msec.as_ref().map_or(0, |value| 1 + sizeof_varint(*(value) as u64))
        + self.event.iter().map(|value| 1 + sizeof_len((value).get_size())).sum::<usize>()
        + self.cell_operator.as_ref().map_or(0, |value| 1 + sizeof_len((value).len()))
        + self.sim_operator.as_ref().map_or(0, |value| 1 + sizeof_len((value).len()))
        + self.roaming.as_ref().map_or(0, |value| 1 + sizeof_len((value).len()))
        + self.user_number.as_ref().map_or(0, |value| 1 + sizeof_varint(*(value) as u64))
        + if self.type_pb == DeviceType::DEVICE_ANDROID_OS { 0 } else { 1 + sizeof_varint(*(&self.type_pb) as u64) }
        + self.chrome_build.as_ref().map_or(0, |value| 1 + sizeof_len((value).get_size()))
    }

    fn write_message<W>(&self, writer: &mut Writer<W>) -> Result<()>
    where
        W: WriterBackend,
    {
        if let Some(ref value) = self.build { writer.write_with_tag(10, |writer| writer.write_message(value))?; }
        if let Some(ref value) = self.last_checkin_msec { writer.write_with_tag(16, |writer| writer.write_int64(*value))?; }
        for value in &self.event { writer.write_with_tag(26, |writer| writer.write_message(value))?; }
        if let Some(ref value) = self.cell_operator { writer.write_with_tag(50, |writer| writer.write_string(&**value))?; }
        if let Some(ref value) = self.sim_operator { writer.write_with_tag(58, |writer| writer.write_string(&**value))?; }
        if let Some(ref value) = self.roaming { writer.write_with_tag(66, |writer| writer.write_string(&**value))?; }
        if let Some(ref value) = self.user_number { writer.write_with_tag(72, |writer| writer.write_int32(*value))?; }
        if self.type_pb != DeviceType::DEVICE_ANDROID_OS { writer.write_with_tag(96, |writer| writer.write_enum(*&self.type_pb as i32))?; }
        if let Some(ref value) = self.chrome_build { writer.write_with_tag(106, |writer| writer.write_message(value))?; }
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

