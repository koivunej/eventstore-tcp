//! Automatically generated rust module for 'ClientMessageDtos.proto' file

#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(dead_code)]
#![allow(unknown_lints)]
#![allow(clippy)]
#![cfg_attr(rustfmt, rustfmt_skip)]

use std::io::Write;
use std::borrow::Cow;
use quick_protobuf::{MessageWrite, BytesReader, Writer, Result};
use quick_protobuf::sizeofs::*;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum OperationResult {
    Success = 0,
    PrepareTimeout = 1,
    CommitTimeout = 2,
    ForwardTimeout = 3,
    WrongExpectedVersion = 4,
    StreamDeleted = 5,
    InvalidTransaction = 6,
    AccessDenied = 7,
}

impl Default for OperationResult {
    fn default() -> Self {
        OperationResult::Success
    }
}

impl From<i32> for OperationResult {
    fn from(i: i32) -> Self {
        match i {
            0 => OperationResult::Success,
            1 => OperationResult::PrepareTimeout,
            2 => OperationResult::CommitTimeout,
            3 => OperationResult::ForwardTimeout,
            4 => OperationResult::WrongExpectedVersion,
            5 => OperationResult::StreamDeleted,
            6 => OperationResult::InvalidTransaction,
            7 => OperationResult::AccessDenied,
            _ => Self::default(),
        }
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct NewEvent<'a> {
    pub event_id: Cow<'a, [u8]>,
    pub event_type: Cow<'a, str>,
    pub data_content_type: i32,
    pub metadata_content_type: i32,
    pub data: Cow<'a, [u8]>,
    pub metadata: Option<Cow<'a, [u8]>>,
}

impl<'a> NewEvent<'a> {
    pub fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.event_id = r.read_bytes(bytes).map(Cow::Borrowed)?,
                Ok(18) => msg.event_type = r.read_string(bytes).map(Cow::Borrowed)?,
                Ok(24) => msg.data_content_type = r.read_int32(bytes)?,
                Ok(32) => msg.metadata_content_type = r.read_int32(bytes)?,
                Ok(42) => msg.data = r.read_bytes(bytes).map(Cow::Borrowed)?,
                Ok(50) => msg.metadata = Some(r.read_bytes(bytes).map(Cow::Borrowed)?),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for NewEvent<'a> {
    fn get_size(&self) -> usize {
        0
        + 1 + sizeof_len((&self.event_id).len())
        + 1 + sizeof_len((&self.event_type).len())
        + 1 + sizeof_varint(*(&self.data_content_type) as u64)
        + 1 + sizeof_varint(*(&self.metadata_content_type) as u64)
        + 1 + sizeof_len((&self.data).len())
        + self.metadata.as_ref().map_or(0, |m| 1 + sizeof_len((m).len()))
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        w.write_with_tag(10, |w| w.write_bytes(&**&self.event_id))?;
        w.write_with_tag(18, |w| w.write_string(&**&self.event_type))?;
        w.write_with_tag(24, |w| w.write_int32(*&self.data_content_type))?;
        w.write_with_tag(32, |w| w.write_int32(*&self.metadata_content_type))?;
        w.write_with_tag(42, |w| w.write_bytes(&**&self.data))?;
        if let Some(ref s) = self.metadata { w.write_with_tag(50, |w| w.write_bytes(&**s))?; }
        Ok(())
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct EventRecord<'a> {
    pub event_stream_id: Cow<'a, str>,
    pub event_number: i32,
    pub event_id: Cow<'a, [u8]>,
    pub event_type: Cow<'a, str>,
    pub data_content_type: i32,
    pub metadata_content_type: i32,
    pub data: Cow<'a, [u8]>,
    pub metadata: Option<Cow<'a, [u8]>>,
    pub created: Option<i64>,
    pub created_epoch: Option<i64>,
}

impl<'a> EventRecord<'a> {
    pub fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.event_stream_id = r.read_string(bytes).map(Cow::Borrowed)?,
                Ok(16) => msg.event_number = r.read_int32(bytes)?,
                Ok(26) => msg.event_id = r.read_bytes(bytes).map(Cow::Borrowed)?,
                Ok(34) => msg.event_type = r.read_string(bytes).map(Cow::Borrowed)?,
                Ok(40) => msg.data_content_type = r.read_int32(bytes)?,
                Ok(48) => msg.metadata_content_type = r.read_int32(bytes)?,
                Ok(58) => msg.data = r.read_bytes(bytes).map(Cow::Borrowed)?,
                Ok(66) => msg.metadata = Some(r.read_bytes(bytes).map(Cow::Borrowed)?),
                Ok(72) => msg.created = Some(r.read_int64(bytes)?),
                Ok(80) => msg.created_epoch = Some(r.read_int64(bytes)?),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for EventRecord<'a> {
    fn get_size(&self) -> usize {
        0
        + 1 + sizeof_len((&self.event_stream_id).len())
        + 1 + sizeof_varint(*(&self.event_number) as u64)
        + 1 + sizeof_len((&self.event_id).len())
        + 1 + sizeof_len((&self.event_type).len())
        + 1 + sizeof_varint(*(&self.data_content_type) as u64)
        + 1 + sizeof_varint(*(&self.metadata_content_type) as u64)
        + 1 + sizeof_len((&self.data).len())
        + self.metadata.as_ref().map_or(0, |m| 1 + sizeof_len((m).len()))
        + self.created.as_ref().map_or(0, |m| 1 + sizeof_varint(*(m) as u64))
        + self.created_epoch.as_ref().map_or(0, |m| 1 + sizeof_varint(*(m) as u64))
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        w.write_with_tag(10, |w| w.write_string(&**&self.event_stream_id))?;
        w.write_with_tag(16, |w| w.write_int32(*&self.event_number))?;
        w.write_with_tag(26, |w| w.write_bytes(&**&self.event_id))?;
        w.write_with_tag(34, |w| w.write_string(&**&self.event_type))?;
        w.write_with_tag(40, |w| w.write_int32(*&self.data_content_type))?;
        w.write_with_tag(48, |w| w.write_int32(*&self.metadata_content_type))?;
        w.write_with_tag(58, |w| w.write_bytes(&**&self.data))?;
        if let Some(ref s) = self.metadata { w.write_with_tag(66, |w| w.write_bytes(&**s))?; }
        if let Some(ref s) = self.created { w.write_with_tag(72, |w| w.write_int64(*s))?; }
        if let Some(ref s) = self.created_epoch { w.write_with_tag(80, |w| w.write_int64(*s))?; }
        Ok(())
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct ResolvedIndexedEvent<'a> {
    pub event: EventRecord<'a>,
    pub link: Option<EventRecord<'a>>,
}

impl<'a> ResolvedIndexedEvent<'a> {
    pub fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.event = r.read_message(bytes, EventRecord::from_reader)?,
                Ok(18) => msg.link = Some(r.read_message(bytes, EventRecord::from_reader)?),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for ResolvedIndexedEvent<'a> {
    fn get_size(&self) -> usize {
        0
        + 1 + sizeof_len((&self.event).get_size())
        + self.link.as_ref().map_or(0, |m| 1 + sizeof_len((m).get_size()))
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        w.write_with_tag(10, |w| w.write_message(&self.event))?;
        if let Some(ref s) = self.link { w.write_with_tag(18, |w| w.write_message(s))?; }
        Ok(())
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct ResolvedEvent<'a> {
    pub event: EventRecord<'a>,
    pub link: Option<EventRecord<'a>>,
    pub commit_position: i64,
    pub prepare_position: i64,
}

impl<'a> ResolvedEvent<'a> {
    pub fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.event = r.read_message(bytes, EventRecord::from_reader)?,
                Ok(18) => msg.link = Some(r.read_message(bytes, EventRecord::from_reader)?),
                Ok(24) => msg.commit_position = r.read_int64(bytes)?,
                Ok(32) => msg.prepare_position = r.read_int64(bytes)?,
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for ResolvedEvent<'a> {
    fn get_size(&self) -> usize {
        0
        + 1 + sizeof_len((&self.event).get_size())
        + self.link.as_ref().map_or(0, |m| 1 + sizeof_len((m).get_size()))
        + 1 + sizeof_varint(*(&self.commit_position) as u64)
        + 1 + sizeof_varint(*(&self.prepare_position) as u64)
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        w.write_with_tag(10, |w| w.write_message(&self.event))?;
        if let Some(ref s) = self.link { w.write_with_tag(18, |w| w.write_message(s))?; }
        w.write_with_tag(24, |w| w.write_int64(*&self.commit_position))?;
        w.write_with_tag(32, |w| w.write_int64(*&self.prepare_position))?;
        Ok(())
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct WriteEvents<'a> {
    pub event_stream_id: Cow<'a, str>,
    pub expected_version: i32,
    pub events: Vec<NewEvent<'a>>,
    pub require_master: bool,
}

impl<'a> WriteEvents<'a> {
    pub fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.event_stream_id = r.read_string(bytes).map(Cow::Borrowed)?,
                Ok(16) => msg.expected_version = r.read_int32(bytes)?,
                Ok(26) => msg.events.push(r.read_message(bytes, NewEvent::from_reader)?),
                Ok(32) => msg.require_master = r.read_bool(bytes)?,
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for WriteEvents<'a> {
    fn get_size(&self) -> usize {
        0
        + 1 + sizeof_len((&self.event_stream_id).len())
        + 1 + sizeof_varint(*(&self.expected_version) as u64)
        + self.events.iter().map(|s| 1 + sizeof_len((s).get_size())).sum::<usize>()
        + 1 + sizeof_varint(*(&self.require_master) as u64)
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        w.write_with_tag(10, |w| w.write_string(&**&self.event_stream_id))?;
        w.write_with_tag(16, |w| w.write_int32(*&self.expected_version))?;
        for s in &self.events { w.write_with_tag(26, |w| w.write_message(s))?; }
        w.write_with_tag(32, |w| w.write_bool(*&self.require_master))?;
        Ok(())
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct WriteEventsCompleted<'a> {
    pub result: Option<OperationResult>,
    pub message: Option<Cow<'a, str>>,
    pub first_event_number: i32,
    pub last_event_number: i32,
    pub prepare_position: Option<i64>,
    pub commit_position: Option<i64>,
}

impl<'a> WriteEventsCompleted<'a> {
    pub fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.result = Some(r.read_enum(bytes)?),
                Ok(18) => msg.message = Some(r.read_string(bytes).map(Cow::Borrowed)?),
                Ok(24) => msg.first_event_number = r.read_int32(bytes)?,
                Ok(32) => msg.last_event_number = r.read_int32(bytes)?,
                Ok(40) => msg.prepare_position = Some(r.read_int64(bytes)?),
                Ok(48) => msg.commit_position = Some(r.read_int64(bytes)?),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for WriteEventsCompleted<'a> {
    fn get_size(&self) -> usize {
        0
        + self.result.as_ref().map_or(0, |m| 1 + sizeof_varint(*(m) as u64))
        + self.message.as_ref().map_or(0, |m| 1 + sizeof_len((m).len()))
        + 1 + sizeof_varint(*(&self.first_event_number) as u64)
        + 1 + sizeof_varint(*(&self.last_event_number) as u64)
        + self.prepare_position.as_ref().map_or(0, |m| 1 + sizeof_varint(*(m) as u64))
        + self.commit_position.as_ref().map_or(0, |m| 1 + sizeof_varint(*(m) as u64))
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        if let Some(ref s) = self.result { w.write_with_tag(8, |w| w.write_enum(*s as i32))?; }
        if let Some(ref s) = self.message { w.write_with_tag(18, |w| w.write_string(&**s))?; }
        w.write_with_tag(24, |w| w.write_int32(*&self.first_event_number))?;
        w.write_with_tag(32, |w| w.write_int32(*&self.last_event_number))?;
        if let Some(ref s) = self.prepare_position { w.write_with_tag(40, |w| w.write_int64(*s))?; }
        if let Some(ref s) = self.commit_position { w.write_with_tag(48, |w| w.write_int64(*s))?; }
        Ok(())
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct DeleteStream<'a> {
    pub event_stream_id: Cow<'a, str>,
    pub expected_version: i32,
    pub require_master: bool,
    pub hard_delete: Option<bool>,
}

impl<'a> DeleteStream<'a> {
    pub fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.event_stream_id = r.read_string(bytes).map(Cow::Borrowed)?,
                Ok(16) => msg.expected_version = r.read_int32(bytes)?,
                Ok(24) => msg.require_master = r.read_bool(bytes)?,
                Ok(32) => msg.hard_delete = Some(r.read_bool(bytes)?),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for DeleteStream<'a> {
    fn get_size(&self) -> usize {
        0
        + 1 + sizeof_len((&self.event_stream_id).len())
        + 1 + sizeof_varint(*(&self.expected_version) as u64)
        + 1 + sizeof_varint(*(&self.require_master) as u64)
        + self.hard_delete.as_ref().map_or(0, |m| 1 + sizeof_varint(*(m) as u64))
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        w.write_with_tag(10, |w| w.write_string(&**&self.event_stream_id))?;
        w.write_with_tag(16, |w| w.write_int32(*&self.expected_version))?;
        w.write_with_tag(24, |w| w.write_bool(*&self.require_master))?;
        if let Some(ref s) = self.hard_delete { w.write_with_tag(32, |w| w.write_bool(*s))?; }
        Ok(())
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct DeleteStreamCompleted<'a> {
    pub result: Option<OperationResult>,
    pub message: Option<Cow<'a, str>>,
    pub prepare_position: Option<i64>,
    pub commit_position: Option<i64>,
}

impl<'a> DeleteStreamCompleted<'a> {
    pub fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.result = Some(r.read_enum(bytes)?),
                Ok(18) => msg.message = Some(r.read_string(bytes).map(Cow::Borrowed)?),
                Ok(24) => msg.prepare_position = Some(r.read_int64(bytes)?),
                Ok(32) => msg.commit_position = Some(r.read_int64(bytes)?),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for DeleteStreamCompleted<'a> {
    fn get_size(&self) -> usize {
        0
        + self.result.as_ref().map_or(0, |m| 1 + sizeof_varint(*(m) as u64))
        + self.message.as_ref().map_or(0, |m| 1 + sizeof_len((m).len()))
        + self.prepare_position.as_ref().map_or(0, |m| 1 + sizeof_varint(*(m) as u64))
        + self.commit_position.as_ref().map_or(0, |m| 1 + sizeof_varint(*(m) as u64))
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        if let Some(ref s) = self.result { w.write_with_tag(8, |w| w.write_enum(*s as i32))?; }
        if let Some(ref s) = self.message { w.write_with_tag(18, |w| w.write_string(&**s))?; }
        if let Some(ref s) = self.prepare_position { w.write_with_tag(24, |w| w.write_int64(*s))?; }
        if let Some(ref s) = self.commit_position { w.write_with_tag(32, |w| w.write_int64(*s))?; }
        Ok(())
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct TransactionStart<'a> {
    pub event_stream_id: Cow<'a, str>,
    pub expected_version: i32,
    pub require_master: bool,
}

impl<'a> TransactionStart<'a> {
    pub fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.event_stream_id = r.read_string(bytes).map(Cow::Borrowed)?,
                Ok(16) => msg.expected_version = r.read_int32(bytes)?,
                Ok(24) => msg.require_master = r.read_bool(bytes)?,
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for TransactionStart<'a> {
    fn get_size(&self) -> usize {
        0
        + 1 + sizeof_len((&self.event_stream_id).len())
        + 1 + sizeof_varint(*(&self.expected_version) as u64)
        + 1 + sizeof_varint(*(&self.require_master) as u64)
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        w.write_with_tag(10, |w| w.write_string(&**&self.event_stream_id))?;
        w.write_with_tag(16, |w| w.write_int32(*&self.expected_version))?;
        w.write_with_tag(24, |w| w.write_bool(*&self.require_master))?;
        Ok(())
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct TransactionStartCompleted<'a> {
    pub transaction_id: i64,
    pub result: Option<OperationResult>,
    pub message: Option<Cow<'a, str>>,
}

impl<'a> TransactionStartCompleted<'a> {
    pub fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.transaction_id = r.read_int64(bytes)?,
                Ok(16) => msg.result = Some(r.read_enum(bytes)?),
                Ok(26) => msg.message = Some(r.read_string(bytes).map(Cow::Borrowed)?),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for TransactionStartCompleted<'a> {
    fn get_size(&self) -> usize {
        0
        + 1 + sizeof_varint(*(&self.transaction_id) as u64)
        + self.result.as_ref().map_or(0, |m| 1 + sizeof_varint(*(m) as u64))
        + self.message.as_ref().map_or(0, |m| 1 + sizeof_len((m).len()))
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        w.write_with_tag(8, |w| w.write_int64(*&self.transaction_id))?;
        if let Some(ref s) = self.result { w.write_with_tag(16, |w| w.write_enum(*s as i32))?; }
        if let Some(ref s) = self.message { w.write_with_tag(26, |w| w.write_string(&**s))?; }
        Ok(())
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct TransactionWrite<'a> {
    pub transaction_id: i64,
    pub events: Vec<NewEvent<'a>>,
    pub require_master: bool,
}

impl<'a> TransactionWrite<'a> {
    pub fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.transaction_id = r.read_int64(bytes)?,
                Ok(18) => msg.events.push(r.read_message(bytes, NewEvent::from_reader)?),
                Ok(24) => msg.require_master = r.read_bool(bytes)?,
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for TransactionWrite<'a> {
    fn get_size(&self) -> usize {
        0
        + 1 + sizeof_varint(*(&self.transaction_id) as u64)
        + self.events.iter().map(|s| 1 + sizeof_len((s).get_size())).sum::<usize>()
        + 1 + sizeof_varint(*(&self.require_master) as u64)
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        w.write_with_tag(8, |w| w.write_int64(*&self.transaction_id))?;
        for s in &self.events { w.write_with_tag(18, |w| w.write_message(s))?; }
        w.write_with_tag(24, |w| w.write_bool(*&self.require_master))?;
        Ok(())
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct TransactionWriteCompleted<'a> {
    pub transaction_id: i64,
    pub result: Option<OperationResult>,
    pub message: Option<Cow<'a, str>>,
}

impl<'a> TransactionWriteCompleted<'a> {
    pub fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.transaction_id = r.read_int64(bytes)?,
                Ok(16) => msg.result = Some(r.read_enum(bytes)?),
                Ok(26) => msg.message = Some(r.read_string(bytes).map(Cow::Borrowed)?),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for TransactionWriteCompleted<'a> {
    fn get_size(&self) -> usize {
        0
        + 1 + sizeof_varint(*(&self.transaction_id) as u64)
        + self.result.as_ref().map_or(0, |m| 1 + sizeof_varint(*(m) as u64))
        + self.message.as_ref().map_or(0, |m| 1 + sizeof_len((m).len()))
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        w.write_with_tag(8, |w| w.write_int64(*&self.transaction_id))?;
        if let Some(ref s) = self.result { w.write_with_tag(16, |w| w.write_enum(*s as i32))?; }
        if let Some(ref s) = self.message { w.write_with_tag(26, |w| w.write_string(&**s))?; }
        Ok(())
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct TransactionCommit {
    pub transaction_id: i64,
    pub require_master: bool,
}

impl TransactionCommit {
    pub fn from_reader(r: &mut BytesReader, bytes: &[u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.transaction_id = r.read_int64(bytes)?,
                Ok(16) => msg.require_master = r.read_bool(bytes)?,
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl MessageWrite for TransactionCommit {
    fn get_size(&self) -> usize {
        0
        + 1 + sizeof_varint(*(&self.transaction_id) as u64)
        + 1 + sizeof_varint(*(&self.require_master) as u64)
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        w.write_with_tag(8, |w| w.write_int64(*&self.transaction_id))?;
        w.write_with_tag(16, |w| w.write_bool(*&self.require_master))?;
        Ok(())
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct TransactionCommitCompleted<'a> {
    pub transaction_id: i64,
    pub result: Option<OperationResult>,
    pub message: Option<Cow<'a, str>>,
    pub first_event_number: i32,
    pub last_event_number: i32,
    pub prepare_position: Option<i64>,
    pub commit_position: Option<i64>,
}

impl<'a> TransactionCommitCompleted<'a> {
    pub fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.transaction_id = r.read_int64(bytes)?,
                Ok(16) => msg.result = Some(r.read_enum(bytes)?),
                Ok(26) => msg.message = Some(r.read_string(bytes).map(Cow::Borrowed)?),
                Ok(32) => msg.first_event_number = r.read_int32(bytes)?,
                Ok(40) => msg.last_event_number = r.read_int32(bytes)?,
                Ok(48) => msg.prepare_position = Some(r.read_int64(bytes)?),
                Ok(56) => msg.commit_position = Some(r.read_int64(bytes)?),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for TransactionCommitCompleted<'a> {
    fn get_size(&self) -> usize {
        0
        + 1 + sizeof_varint(*(&self.transaction_id) as u64)
        + self.result.as_ref().map_or(0, |m| 1 + sizeof_varint(*(m) as u64))
        + self.message.as_ref().map_or(0, |m| 1 + sizeof_len((m).len()))
        + 1 + sizeof_varint(*(&self.first_event_number) as u64)
        + 1 + sizeof_varint(*(&self.last_event_number) as u64)
        + self.prepare_position.as_ref().map_or(0, |m| 1 + sizeof_varint(*(m) as u64))
        + self.commit_position.as_ref().map_or(0, |m| 1 + sizeof_varint(*(m) as u64))
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        w.write_with_tag(8, |w| w.write_int64(*&self.transaction_id))?;
        if let Some(ref s) = self.result { w.write_with_tag(16, |w| w.write_enum(*s as i32))?; }
        if let Some(ref s) = self.message { w.write_with_tag(26, |w| w.write_string(&**s))?; }
        w.write_with_tag(32, |w| w.write_int32(*&self.first_event_number))?;
        w.write_with_tag(40, |w| w.write_int32(*&self.last_event_number))?;
        if let Some(ref s) = self.prepare_position { w.write_with_tag(48, |w| w.write_int64(*s))?; }
        if let Some(ref s) = self.commit_position { w.write_with_tag(56, |w| w.write_int64(*s))?; }
        Ok(())
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct ReadEvent<'a> {
    pub event_stream_id: Cow<'a, str>,
    pub event_number: i32,
    pub resolve_link_tos: bool,
    pub require_master: bool,
}

impl<'a> ReadEvent<'a> {
    pub fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.event_stream_id = r.read_string(bytes).map(Cow::Borrowed)?,
                Ok(16) => msg.event_number = r.read_int32(bytes)?,
                Ok(24) => msg.resolve_link_tos = r.read_bool(bytes)?,
                Ok(32) => msg.require_master = r.read_bool(bytes)?,
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for ReadEvent<'a> {
    fn get_size(&self) -> usize {
        0
        + 1 + sizeof_len((&self.event_stream_id).len())
        + 1 + sizeof_varint(*(&self.event_number) as u64)
        + 1 + sizeof_varint(*(&self.resolve_link_tos) as u64)
        + 1 + sizeof_varint(*(&self.require_master) as u64)
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        w.write_with_tag(10, |w| w.write_string(&**&self.event_stream_id))?;
        w.write_with_tag(16, |w| w.write_int32(*&self.event_number))?;
        w.write_with_tag(24, |w| w.write_bool(*&self.resolve_link_tos))?;
        w.write_with_tag(32, |w| w.write_bool(*&self.require_master))?;
        Ok(())
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct ReadEventCompleted<'a> {
    pub result: Option<mod_ReadEventCompleted::ReadEventResult>,
    pub event: ResolvedIndexedEvent<'a>,
    pub error: Option<Cow<'a, str>>,
}

impl<'a> ReadEventCompleted<'a> {
    pub fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.result = Some(r.read_enum(bytes)?),
                Ok(18) => msg.event = r.read_message(bytes, ResolvedIndexedEvent::from_reader)?,
                Ok(26) => msg.error = Some(r.read_string(bytes).map(Cow::Borrowed)?),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for ReadEventCompleted<'a> {
    fn get_size(&self) -> usize {
        0
        + self.result.as_ref().map_or(0, |m| 1 + sizeof_varint(*(m) as u64))
        + 1 + sizeof_len((&self.event).get_size())
        + self.error.as_ref().map_or(0, |m| 1 + sizeof_len((m).len()))
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        if let Some(ref s) = self.result { w.write_with_tag(8, |w| w.write_enum(*s as i32))?; }
        w.write_with_tag(18, |w| w.write_message(&self.event))?;
        if let Some(ref s) = self.error { w.write_with_tag(26, |w| w.write_string(&**s))?; }
        Ok(())
    }
}

pub mod mod_ReadEventCompleted {


#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ReadEventResult {
    Success = 0,
    NotFound = 1,
    NoStream = 2,
    StreamDeleted = 3,
    Error = 4,
    AccessDenied = 5,
}

impl Default for ReadEventResult {
    fn default() -> Self {
        ReadEventResult::Success
    }
}

impl From<i32> for ReadEventResult {
    fn from(i: i32) -> Self {
        match i {
            0 => ReadEventResult::Success,
            1 => ReadEventResult::NotFound,
            2 => ReadEventResult::NoStream,
            3 => ReadEventResult::StreamDeleted,
            4 => ReadEventResult::Error,
            5 => ReadEventResult::AccessDenied,
            _ => Self::default(),
        }
    }
}

}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct ReadStreamEvents<'a> {
    pub event_stream_id: Cow<'a, str>,
    pub from_event_number: i32,
    pub max_count: i32,
    pub resolve_link_tos: bool,
    pub require_master: bool,
}

impl<'a> ReadStreamEvents<'a> {
    pub fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.event_stream_id = r.read_string(bytes).map(Cow::Borrowed)?,
                Ok(16) => msg.from_event_number = r.read_int32(bytes)?,
                Ok(24) => msg.max_count = r.read_int32(bytes)?,
                Ok(32) => msg.resolve_link_tos = r.read_bool(bytes)?,
                Ok(40) => msg.require_master = r.read_bool(bytes)?,
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for ReadStreamEvents<'a> {
    fn get_size(&self) -> usize {
        0
        + 1 + sizeof_len((&self.event_stream_id).len())
        + 1 + sizeof_varint(*(&self.from_event_number) as u64)
        + 1 + sizeof_varint(*(&self.max_count) as u64)
        + 1 + sizeof_varint(*(&self.resolve_link_tos) as u64)
        + 1 + sizeof_varint(*(&self.require_master) as u64)
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        w.write_with_tag(10, |w| w.write_string(&**&self.event_stream_id))?;
        w.write_with_tag(16, |w| w.write_int32(*&self.from_event_number))?;
        w.write_with_tag(24, |w| w.write_int32(*&self.max_count))?;
        w.write_with_tag(32, |w| w.write_bool(*&self.resolve_link_tos))?;
        w.write_with_tag(40, |w| w.write_bool(*&self.require_master))?;
        Ok(())
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct ReadStreamEventsCompleted<'a> {
    pub events: Vec<ResolvedIndexedEvent<'a>>,
    pub result: Option<mod_ReadStreamEventsCompleted::ReadStreamResult>,
    pub next_event_number: i32,
    pub last_event_number: i32,
    pub is_end_of_stream: bool,
    pub last_commit_position: i64,
    pub error: Option<Cow<'a, str>>,
}

impl<'a> ReadStreamEventsCompleted<'a> {
    pub fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.events.push(r.read_message(bytes, ResolvedIndexedEvent::from_reader)?),
                Ok(16) => msg.result = Some(r.read_enum(bytes)?),
                Ok(24) => msg.next_event_number = r.read_int32(bytes)?,
                Ok(32) => msg.last_event_number = r.read_int32(bytes)?,
                Ok(40) => msg.is_end_of_stream = r.read_bool(bytes)?,
                Ok(48) => msg.last_commit_position = r.read_int64(bytes)?,
                Ok(58) => msg.error = Some(r.read_string(bytes).map(Cow::Borrowed)?),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for ReadStreamEventsCompleted<'a> {
    fn get_size(&self) -> usize {
        0
        + self.events.iter().map(|s| 1 + sizeof_len((s).get_size())).sum::<usize>()
        + self.result.as_ref().map_or(0, |m| 1 + sizeof_varint(*(m) as u64))
        + 1 + sizeof_varint(*(&self.next_event_number) as u64)
        + 1 + sizeof_varint(*(&self.last_event_number) as u64)
        + 1 + sizeof_varint(*(&self.is_end_of_stream) as u64)
        + 1 + sizeof_varint(*(&self.last_commit_position) as u64)
        + self.error.as_ref().map_or(0, |m| 1 + sizeof_len((m).len()))
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        for s in &self.events { w.write_with_tag(10, |w| w.write_message(s))?; }
        if let Some(ref s) = self.result { w.write_with_tag(16, |w| w.write_enum(*s as i32))?; }
        w.write_with_tag(24, |w| w.write_int32(*&self.next_event_number))?;
        w.write_with_tag(32, |w| w.write_int32(*&self.last_event_number))?;
        w.write_with_tag(40, |w| w.write_bool(*&self.is_end_of_stream))?;
        w.write_with_tag(48, |w| w.write_int64(*&self.last_commit_position))?;
        if let Some(ref s) = self.error { w.write_with_tag(58, |w| w.write_string(&**s))?; }
        Ok(())
    }
}

pub mod mod_ReadStreamEventsCompleted {


#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ReadStreamResult {
    Success = 0,
    NoStream = 1,
    StreamDeleted = 2,
    NotModified = 3,
    Error = 4,
    AccessDenied = 5,
}

impl Default for ReadStreamResult {
    fn default() -> Self {
        ReadStreamResult::Success
    }
}

impl From<i32> for ReadStreamResult {
    fn from(i: i32) -> Self {
        match i {
            0 => ReadStreamResult::Success,
            1 => ReadStreamResult::NoStream,
            2 => ReadStreamResult::StreamDeleted,
            3 => ReadStreamResult::NotModified,
            4 => ReadStreamResult::Error,
            5 => ReadStreamResult::AccessDenied,
            _ => Self::default(),
        }
    }
}

}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct ReadAllEvents {
    pub commit_position: i64,
    pub prepare_position: i64,
    pub max_count: i32,
    pub resolve_link_tos: bool,
    pub require_master: bool,
}

impl ReadAllEvents {
    pub fn from_reader(r: &mut BytesReader, bytes: &[u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.commit_position = r.read_int64(bytes)?,
                Ok(16) => msg.prepare_position = r.read_int64(bytes)?,
                Ok(24) => msg.max_count = r.read_int32(bytes)?,
                Ok(32) => msg.resolve_link_tos = r.read_bool(bytes)?,
                Ok(40) => msg.require_master = r.read_bool(bytes)?,
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl MessageWrite for ReadAllEvents {
    fn get_size(&self) -> usize {
        0
        + 1 + sizeof_varint(*(&self.commit_position) as u64)
        + 1 + sizeof_varint(*(&self.prepare_position) as u64)
        + 1 + sizeof_varint(*(&self.max_count) as u64)
        + 1 + sizeof_varint(*(&self.resolve_link_tos) as u64)
        + 1 + sizeof_varint(*(&self.require_master) as u64)
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        w.write_with_tag(8, |w| w.write_int64(*&self.commit_position))?;
        w.write_with_tag(16, |w| w.write_int64(*&self.prepare_position))?;
        w.write_with_tag(24, |w| w.write_int32(*&self.max_count))?;
        w.write_with_tag(32, |w| w.write_bool(*&self.resolve_link_tos))?;
        w.write_with_tag(40, |w| w.write_bool(*&self.require_master))?;
        Ok(())
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct ReadAllEventsCompleted<'a> {
    pub commit_position: i64,
    pub prepare_position: i64,
    pub events: Vec<ResolvedEvent<'a>>,
    pub next_commit_position: i64,
    pub next_prepare_position: i64,
    pub result: mod_ReadAllEventsCompleted::ReadAllResult,
    pub error: Option<Cow<'a, str>>,
}

impl<'a> ReadAllEventsCompleted<'a> {
    pub fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = ReadAllEventsCompleted {
            result: mod_ReadAllEventsCompleted::ReadAllResult::Success,
            ..Self::default()
        };
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.commit_position = r.read_int64(bytes)?,
                Ok(16) => msg.prepare_position = r.read_int64(bytes)?,
                Ok(26) => msg.events.push(r.read_message(bytes, ResolvedEvent::from_reader)?),
                Ok(32) => msg.next_commit_position = r.read_int64(bytes)?,
                Ok(40) => msg.next_prepare_position = r.read_int64(bytes)?,
                Ok(48) => msg.result = r.read_enum(bytes)?,
                Ok(58) => msg.error = Some(r.read_string(bytes).map(Cow::Borrowed)?),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for ReadAllEventsCompleted<'a> {
    fn get_size(&self) -> usize {
        0
        + 1 + sizeof_varint(*(&self.commit_position) as u64)
        + 1 + sizeof_varint(*(&self.prepare_position) as u64)
        + self.events.iter().map(|s| 1 + sizeof_len((s).get_size())).sum::<usize>()
        + 1 + sizeof_varint(*(&self.next_commit_position) as u64)
        + 1 + sizeof_varint(*(&self.next_prepare_position) as u64)
        + if self.result == mod_ReadAllEventsCompleted::ReadAllResult::Success { 0 } else { 1 + sizeof_varint(*(&self.result) as u64) }
        + self.error.as_ref().map_or(0, |m| 1 + sizeof_len((m).len()))
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        w.write_with_tag(8, |w| w.write_int64(*&self.commit_position))?;
        w.write_with_tag(16, |w| w.write_int64(*&self.prepare_position))?;
        for s in &self.events { w.write_with_tag(26, |w| w.write_message(s))?; }
        w.write_with_tag(32, |w| w.write_int64(*&self.next_commit_position))?;
        w.write_with_tag(40, |w| w.write_int64(*&self.next_prepare_position))?;
        if self.result != mod_ReadAllEventsCompleted::ReadAllResult::Success { w.write_with_tag(48, |w| w.write_enum(*&self.result as i32))?; }
        if let Some(ref s) = self.error { w.write_with_tag(58, |w| w.write_string(&**s))?; }
        Ok(())
    }
}

pub mod mod_ReadAllEventsCompleted {


#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ReadAllResult {
    Success = 0,
    NotModified = 1,
    Error = 2,
    AccessDenied = 3,
}

impl Default for ReadAllResult {
    fn default() -> Self {
        ReadAllResult::Success
    }
}

impl From<i32> for ReadAllResult {
    fn from(i: i32) -> Self {
        match i {
            0 => ReadAllResult::Success,
            1 => ReadAllResult::NotModified,
            2 => ReadAllResult::Error,
            3 => ReadAllResult::AccessDenied,
            _ => Self::default(),
        }
    }
}

}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct CreatePersistentSubscription<'a> {
    pub subscription_group_name: Cow<'a, str>,
    pub event_stream_id: Cow<'a, str>,
    pub resolve_link_tos: bool,
    pub start_from: i32,
    pub message_timeout_milliseconds: i32,
    pub record_statistics: bool,
    pub live_buffer_size: i32,
    pub read_batch_size: i32,
    pub buffer_size: i32,
    pub max_retry_count: i32,
    pub prefer_round_robin: bool,
    pub checkpoint_after_time: i32,
    pub checkpoint_max_count: i32,
    pub checkpoint_min_count: i32,
    pub subscriber_max_count: i32,
    pub named_consumer_strategy: Option<Cow<'a, str>>,
}

impl<'a> CreatePersistentSubscription<'a> {
    pub fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.subscription_group_name = r.read_string(bytes).map(Cow::Borrowed)?,
                Ok(18) => msg.event_stream_id = r.read_string(bytes).map(Cow::Borrowed)?,
                Ok(24) => msg.resolve_link_tos = r.read_bool(bytes)?,
                Ok(32) => msg.start_from = r.read_int32(bytes)?,
                Ok(40) => msg.message_timeout_milliseconds = r.read_int32(bytes)?,
                Ok(48) => msg.record_statistics = r.read_bool(bytes)?,
                Ok(56) => msg.live_buffer_size = r.read_int32(bytes)?,
                Ok(64) => msg.read_batch_size = r.read_int32(bytes)?,
                Ok(72) => msg.buffer_size = r.read_int32(bytes)?,
                Ok(80) => msg.max_retry_count = r.read_int32(bytes)?,
                Ok(88) => msg.prefer_round_robin = r.read_bool(bytes)?,
                Ok(96) => msg.checkpoint_after_time = r.read_int32(bytes)?,
                Ok(104) => msg.checkpoint_max_count = r.read_int32(bytes)?,
                Ok(112) => msg.checkpoint_min_count = r.read_int32(bytes)?,
                Ok(120) => msg.subscriber_max_count = r.read_int32(bytes)?,
                Ok(130) => msg.named_consumer_strategy = Some(r.read_string(bytes).map(Cow::Borrowed)?),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for CreatePersistentSubscription<'a> {
    fn get_size(&self) -> usize {
        0
        + 1 + sizeof_len((&self.subscription_group_name).len())
        + 1 + sizeof_len((&self.event_stream_id).len())
        + 1 + sizeof_varint(*(&self.resolve_link_tos) as u64)
        + 1 + sizeof_varint(*(&self.start_from) as u64)
        + 1 + sizeof_varint(*(&self.message_timeout_milliseconds) as u64)
        + 1 + sizeof_varint(*(&self.record_statistics) as u64)
        + 1 + sizeof_varint(*(&self.live_buffer_size) as u64)
        + 1 + sizeof_varint(*(&self.read_batch_size) as u64)
        + 1 + sizeof_varint(*(&self.buffer_size) as u64)
        + 1 + sizeof_varint(*(&self.max_retry_count) as u64)
        + 1 + sizeof_varint(*(&self.prefer_round_robin) as u64)
        + 1 + sizeof_varint(*(&self.checkpoint_after_time) as u64)
        + 1 + sizeof_varint(*(&self.checkpoint_max_count) as u64)
        + 1 + sizeof_varint(*(&self.checkpoint_min_count) as u64)
        + 1 + sizeof_varint(*(&self.subscriber_max_count) as u64)
        + self.named_consumer_strategy.as_ref().map_or(0, |m| 2 + sizeof_len((m).len()))
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        w.write_with_tag(10, |w| w.write_string(&**&self.subscription_group_name))?;
        w.write_with_tag(18, |w| w.write_string(&**&self.event_stream_id))?;
        w.write_with_tag(24, |w| w.write_bool(*&self.resolve_link_tos))?;
        w.write_with_tag(32, |w| w.write_int32(*&self.start_from))?;
        w.write_with_tag(40, |w| w.write_int32(*&self.message_timeout_milliseconds))?;
        w.write_with_tag(48, |w| w.write_bool(*&self.record_statistics))?;
        w.write_with_tag(56, |w| w.write_int32(*&self.live_buffer_size))?;
        w.write_with_tag(64, |w| w.write_int32(*&self.read_batch_size))?;
        w.write_with_tag(72, |w| w.write_int32(*&self.buffer_size))?;
        w.write_with_tag(80, |w| w.write_int32(*&self.max_retry_count))?;
        w.write_with_tag(88, |w| w.write_bool(*&self.prefer_round_robin))?;
        w.write_with_tag(96, |w| w.write_int32(*&self.checkpoint_after_time))?;
        w.write_with_tag(104, |w| w.write_int32(*&self.checkpoint_max_count))?;
        w.write_with_tag(112, |w| w.write_int32(*&self.checkpoint_min_count))?;
        w.write_with_tag(120, |w| w.write_int32(*&self.subscriber_max_count))?;
        if let Some(ref s) = self.named_consumer_strategy { w.write_with_tag(130, |w| w.write_string(&**s))?; }
        Ok(())
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct DeletePersistentSubscription<'a> {
    pub subscription_group_name: Cow<'a, str>,
    pub event_stream_id: Cow<'a, str>,
}

impl<'a> DeletePersistentSubscription<'a> {
    pub fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.subscription_group_name = r.read_string(bytes).map(Cow::Borrowed)?,
                Ok(18) => msg.event_stream_id = r.read_string(bytes).map(Cow::Borrowed)?,
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for DeletePersistentSubscription<'a> {
    fn get_size(&self) -> usize {
        0
        + 1 + sizeof_len((&self.subscription_group_name).len())
        + 1 + sizeof_len((&self.event_stream_id).len())
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        w.write_with_tag(10, |w| w.write_string(&**&self.subscription_group_name))?;
        w.write_with_tag(18, |w| w.write_string(&**&self.event_stream_id))?;
        Ok(())
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct UpdatePersistentSubscription<'a> {
    pub subscription_group_name: Cow<'a, str>,
    pub event_stream_id: Cow<'a, str>,
    pub resolve_link_tos: bool,
    pub start_from: i32,
    pub message_timeout_milliseconds: i32,
    pub record_statistics: bool,
    pub live_buffer_size: i32,
    pub read_batch_size: i32,
    pub buffer_size: i32,
    pub max_retry_count: i32,
    pub prefer_round_robin: bool,
    pub checkpoint_after_time: i32,
    pub checkpoint_max_count: i32,
    pub checkpoint_min_count: i32,
    pub subscriber_max_count: i32,
    pub named_consumer_strategy: Option<Cow<'a, str>>,
}

impl<'a> UpdatePersistentSubscription<'a> {
    pub fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.subscription_group_name = r.read_string(bytes).map(Cow::Borrowed)?,
                Ok(18) => msg.event_stream_id = r.read_string(bytes).map(Cow::Borrowed)?,
                Ok(24) => msg.resolve_link_tos = r.read_bool(bytes)?,
                Ok(32) => msg.start_from = r.read_int32(bytes)?,
                Ok(40) => msg.message_timeout_milliseconds = r.read_int32(bytes)?,
                Ok(48) => msg.record_statistics = r.read_bool(bytes)?,
                Ok(56) => msg.live_buffer_size = r.read_int32(bytes)?,
                Ok(64) => msg.read_batch_size = r.read_int32(bytes)?,
                Ok(72) => msg.buffer_size = r.read_int32(bytes)?,
                Ok(80) => msg.max_retry_count = r.read_int32(bytes)?,
                Ok(88) => msg.prefer_round_robin = r.read_bool(bytes)?,
                Ok(96) => msg.checkpoint_after_time = r.read_int32(bytes)?,
                Ok(104) => msg.checkpoint_max_count = r.read_int32(bytes)?,
                Ok(112) => msg.checkpoint_min_count = r.read_int32(bytes)?,
                Ok(120) => msg.subscriber_max_count = r.read_int32(bytes)?,
                Ok(130) => msg.named_consumer_strategy = Some(r.read_string(bytes).map(Cow::Borrowed)?),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for UpdatePersistentSubscription<'a> {
    fn get_size(&self) -> usize {
        0
        + 1 + sizeof_len((&self.subscription_group_name).len())
        + 1 + sizeof_len((&self.event_stream_id).len())
        + 1 + sizeof_varint(*(&self.resolve_link_tos) as u64)
        + 1 + sizeof_varint(*(&self.start_from) as u64)
        + 1 + sizeof_varint(*(&self.message_timeout_milliseconds) as u64)
        + 1 + sizeof_varint(*(&self.record_statistics) as u64)
        + 1 + sizeof_varint(*(&self.live_buffer_size) as u64)
        + 1 + sizeof_varint(*(&self.read_batch_size) as u64)
        + 1 + sizeof_varint(*(&self.buffer_size) as u64)
        + 1 + sizeof_varint(*(&self.max_retry_count) as u64)
        + 1 + sizeof_varint(*(&self.prefer_round_robin) as u64)
        + 1 + sizeof_varint(*(&self.checkpoint_after_time) as u64)
        + 1 + sizeof_varint(*(&self.checkpoint_max_count) as u64)
        + 1 + sizeof_varint(*(&self.checkpoint_min_count) as u64)
        + 1 + sizeof_varint(*(&self.subscriber_max_count) as u64)
        + self.named_consumer_strategy.as_ref().map_or(0, |m| 2 + sizeof_len((m).len()))
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        w.write_with_tag(10, |w| w.write_string(&**&self.subscription_group_name))?;
        w.write_with_tag(18, |w| w.write_string(&**&self.event_stream_id))?;
        w.write_with_tag(24, |w| w.write_bool(*&self.resolve_link_tos))?;
        w.write_with_tag(32, |w| w.write_int32(*&self.start_from))?;
        w.write_with_tag(40, |w| w.write_int32(*&self.message_timeout_milliseconds))?;
        w.write_with_tag(48, |w| w.write_bool(*&self.record_statistics))?;
        w.write_with_tag(56, |w| w.write_int32(*&self.live_buffer_size))?;
        w.write_with_tag(64, |w| w.write_int32(*&self.read_batch_size))?;
        w.write_with_tag(72, |w| w.write_int32(*&self.buffer_size))?;
        w.write_with_tag(80, |w| w.write_int32(*&self.max_retry_count))?;
        w.write_with_tag(88, |w| w.write_bool(*&self.prefer_round_robin))?;
        w.write_with_tag(96, |w| w.write_int32(*&self.checkpoint_after_time))?;
        w.write_with_tag(104, |w| w.write_int32(*&self.checkpoint_max_count))?;
        w.write_with_tag(112, |w| w.write_int32(*&self.checkpoint_min_count))?;
        w.write_with_tag(120, |w| w.write_int32(*&self.subscriber_max_count))?;
        if let Some(ref s) = self.named_consumer_strategy { w.write_with_tag(130, |w| w.write_string(&**s))?; }
        Ok(())
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct UpdatePersistentSubscriptionCompleted<'a> {
    pub result: mod_UpdatePersistentSubscriptionCompleted::UpdatePersistentSubscriptionResult,
    pub reason: Option<Cow<'a, str>>,
}

impl<'a> UpdatePersistentSubscriptionCompleted<'a> {
    pub fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = UpdatePersistentSubscriptionCompleted {
            result: mod_UpdatePersistentSubscriptionCompleted::UpdatePersistentSubscriptionResult::Success,
            ..Self::default()
        };
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.result = r.read_enum(bytes)?,
                Ok(18) => msg.reason = Some(r.read_string(bytes).map(Cow::Borrowed)?),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for UpdatePersistentSubscriptionCompleted<'a> {
    fn get_size(&self) -> usize {
        0
        + if self.result == mod_UpdatePersistentSubscriptionCompleted::UpdatePersistentSubscriptionResult::Success { 0 } else { 1 + sizeof_varint(*(&self.result) as u64) }
        + self.reason.as_ref().map_or(0, |m| 1 + sizeof_len((m).len()))
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        if self.result != mod_UpdatePersistentSubscriptionCompleted::UpdatePersistentSubscriptionResult::Success { w.write_with_tag(8, |w| w.write_enum(*&self.result as i32))?; }
        if let Some(ref s) = self.reason { w.write_with_tag(18, |w| w.write_string(&**s))?; }
        Ok(())
    }
}

pub mod mod_UpdatePersistentSubscriptionCompleted {


#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum UpdatePersistentSubscriptionResult {
    Success = 0,
    DoesNotExist = 1,
    Fail = 2,
    AccessDenied = 3,
}

impl Default for UpdatePersistentSubscriptionResult {
    fn default() -> Self {
        UpdatePersistentSubscriptionResult::Success
    }
}

impl From<i32> for UpdatePersistentSubscriptionResult {
    fn from(i: i32) -> Self {
        match i {
            0 => UpdatePersistentSubscriptionResult::Success,
            1 => UpdatePersistentSubscriptionResult::DoesNotExist,
            2 => UpdatePersistentSubscriptionResult::Fail,
            3 => UpdatePersistentSubscriptionResult::AccessDenied,
            _ => Self::default(),
        }
    }
}

}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct CreatePersistentSubscriptionCompleted<'a> {
    pub result: mod_CreatePersistentSubscriptionCompleted::CreatePersistentSubscriptionResult,
    pub reason: Option<Cow<'a, str>>,
}

impl<'a> CreatePersistentSubscriptionCompleted<'a> {
    pub fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = CreatePersistentSubscriptionCompleted {
            result: mod_CreatePersistentSubscriptionCompleted::CreatePersistentSubscriptionResult::Success,
            ..Self::default()
        };
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.result = r.read_enum(bytes)?,
                Ok(18) => msg.reason = Some(r.read_string(bytes).map(Cow::Borrowed)?),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for CreatePersistentSubscriptionCompleted<'a> {
    fn get_size(&self) -> usize {
        0
        + if self.result == mod_CreatePersistentSubscriptionCompleted::CreatePersistentSubscriptionResult::Success { 0 } else { 1 + sizeof_varint(*(&self.result) as u64) }
        + self.reason.as_ref().map_or(0, |m| 1 + sizeof_len((m).len()))
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        if self.result != mod_CreatePersistentSubscriptionCompleted::CreatePersistentSubscriptionResult::Success { w.write_with_tag(8, |w| w.write_enum(*&self.result as i32))?; }
        if let Some(ref s) = self.reason { w.write_with_tag(18, |w| w.write_string(&**s))?; }
        Ok(())
    }
}

pub mod mod_CreatePersistentSubscriptionCompleted {


#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum CreatePersistentSubscriptionResult {
    Success = 0,
    AlreadyExists = 1,
    Fail = 2,
    AccessDenied = 3,
}

impl Default for CreatePersistentSubscriptionResult {
    fn default() -> Self {
        CreatePersistentSubscriptionResult::Success
    }
}

impl From<i32> for CreatePersistentSubscriptionResult {
    fn from(i: i32) -> Self {
        match i {
            0 => CreatePersistentSubscriptionResult::Success,
            1 => CreatePersistentSubscriptionResult::AlreadyExists,
            2 => CreatePersistentSubscriptionResult::Fail,
            3 => CreatePersistentSubscriptionResult::AccessDenied,
            _ => Self::default(),
        }
    }
}

}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct DeletePersistentSubscriptionCompleted<'a> {
    pub result: mod_DeletePersistentSubscriptionCompleted::DeletePersistentSubscriptionResult,
    pub reason: Option<Cow<'a, str>>,
}

impl<'a> DeletePersistentSubscriptionCompleted<'a> {
    pub fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = DeletePersistentSubscriptionCompleted {
            result: mod_DeletePersistentSubscriptionCompleted::DeletePersistentSubscriptionResult::Success,
            ..Self::default()
        };
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.result = r.read_enum(bytes)?,
                Ok(18) => msg.reason = Some(r.read_string(bytes).map(Cow::Borrowed)?),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for DeletePersistentSubscriptionCompleted<'a> {
    fn get_size(&self) -> usize {
        0
        + if self.result == mod_DeletePersistentSubscriptionCompleted::DeletePersistentSubscriptionResult::Success { 0 } else { 1 + sizeof_varint(*(&self.result) as u64) }
        + self.reason.as_ref().map_or(0, |m| 1 + sizeof_len((m).len()))
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        if self.result != mod_DeletePersistentSubscriptionCompleted::DeletePersistentSubscriptionResult::Success { w.write_with_tag(8, |w| w.write_enum(*&self.result as i32))?; }
        if let Some(ref s) = self.reason { w.write_with_tag(18, |w| w.write_string(&**s))?; }
        Ok(())
    }
}

pub mod mod_DeletePersistentSubscriptionCompleted {


#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum DeletePersistentSubscriptionResult {
    Success = 0,
    DoesNotExist = 1,
    Fail = 2,
    AccessDenied = 3,
}

impl Default for DeletePersistentSubscriptionResult {
    fn default() -> Self {
        DeletePersistentSubscriptionResult::Success
    }
}

impl From<i32> for DeletePersistentSubscriptionResult {
    fn from(i: i32) -> Self {
        match i {
            0 => DeletePersistentSubscriptionResult::Success,
            1 => DeletePersistentSubscriptionResult::DoesNotExist,
            2 => DeletePersistentSubscriptionResult::Fail,
            3 => DeletePersistentSubscriptionResult::AccessDenied,
            _ => Self::default(),
        }
    }
}

}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct ConnectToPersistentSubscription<'a> {
    pub subscription_id: Cow<'a, str>,
    pub event_stream_id: Cow<'a, str>,
    pub allowed_in_flight_messages: i32,
}

impl<'a> ConnectToPersistentSubscription<'a> {
    pub fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.subscription_id = r.read_string(bytes).map(Cow::Borrowed)?,
                Ok(18) => msg.event_stream_id = r.read_string(bytes).map(Cow::Borrowed)?,
                Ok(24) => msg.allowed_in_flight_messages = r.read_int32(bytes)?,
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for ConnectToPersistentSubscription<'a> {
    fn get_size(&self) -> usize {
        0
        + 1 + sizeof_len((&self.subscription_id).len())
        + 1 + sizeof_len((&self.event_stream_id).len())
        + 1 + sizeof_varint(*(&self.allowed_in_flight_messages) as u64)
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        w.write_with_tag(10, |w| w.write_string(&**&self.subscription_id))?;
        w.write_with_tag(18, |w| w.write_string(&**&self.event_stream_id))?;
        w.write_with_tag(24, |w| w.write_int32(*&self.allowed_in_flight_messages))?;
        Ok(())
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct PersistentSubscriptionAckEvents<'a> {
    pub subscription_id: Cow<'a, str>,
    pub processed_event_ids: Vec<Cow<'a, [u8]>>,
}

impl<'a> PersistentSubscriptionAckEvents<'a> {
    pub fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.subscription_id = r.read_string(bytes).map(Cow::Borrowed)?,
                Ok(18) => msg.processed_event_ids.push(r.read_bytes(bytes).map(Cow::Borrowed)?),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for PersistentSubscriptionAckEvents<'a> {
    fn get_size(&self) -> usize {
        0
        + 1 + sizeof_len((&self.subscription_id).len())
        + self.processed_event_ids.iter().map(|s| 1 + sizeof_len((s).len())).sum::<usize>()
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        w.write_with_tag(10, |w| w.write_string(&**&self.subscription_id))?;
        for s in &self.processed_event_ids { w.write_with_tag(18, |w| w.write_bytes(&**s))?; }
        Ok(())
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct PersistentSubscriptionNakEvents<'a> {
    pub subscription_id: Cow<'a, str>,
    pub processed_event_ids: Vec<Cow<'a, [u8]>>,
    pub message: Option<Cow<'a, str>>,
    pub action: mod_PersistentSubscriptionNakEvents::NakAction,
}

impl<'a> PersistentSubscriptionNakEvents<'a> {
    pub fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = PersistentSubscriptionNakEvents {
            action: mod_PersistentSubscriptionNakEvents::NakAction::Unknown,
            ..Self::default()
        };
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.subscription_id = r.read_string(bytes).map(Cow::Borrowed)?,
                Ok(18) => msg.processed_event_ids.push(r.read_bytes(bytes).map(Cow::Borrowed)?),
                Ok(26) => msg.message = Some(r.read_string(bytes).map(Cow::Borrowed)?),
                Ok(32) => msg.action = r.read_enum(bytes)?,
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for PersistentSubscriptionNakEvents<'a> {
    fn get_size(&self) -> usize {
        0
        + 1 + sizeof_len((&self.subscription_id).len())
        + self.processed_event_ids.iter().map(|s| 1 + sizeof_len((s).len())).sum::<usize>()
        + self.message.as_ref().map_or(0, |m| 1 + sizeof_len((m).len()))
        + if self.action == mod_PersistentSubscriptionNakEvents::NakAction::Unknown { 0 } else { 1 + sizeof_varint(*(&self.action) as u64) }
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        w.write_with_tag(10, |w| w.write_string(&**&self.subscription_id))?;
        for s in &self.processed_event_ids { w.write_with_tag(18, |w| w.write_bytes(&**s))?; }
        if let Some(ref s) = self.message { w.write_with_tag(26, |w| w.write_string(&**s))?; }
        if self.action != mod_PersistentSubscriptionNakEvents::NakAction::Unknown { w.write_with_tag(32, |w| w.write_enum(*&self.action as i32))?; }
        Ok(())
    }
}

pub mod mod_PersistentSubscriptionNakEvents {


#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum NakAction {
    Unknown = 0,
    Park = 1,
    Retry = 2,
    Skip = 3,
    Stop = 4,
}

impl Default for NakAction {
    fn default() -> Self {
        NakAction::Unknown
    }
}

impl From<i32> for NakAction {
    fn from(i: i32) -> Self {
        match i {
            0 => NakAction::Unknown,
            1 => NakAction::Park,
            2 => NakAction::Retry,
            3 => NakAction::Skip,
            4 => NakAction::Stop,
            _ => Self::default(),
        }
    }
}

}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct PersistentSubscriptionConfirmation<'a> {
    pub last_commit_position: i64,
    pub subscription_id: Cow<'a, str>,
    pub last_event_number: Option<i32>,
}

impl<'a> PersistentSubscriptionConfirmation<'a> {
    pub fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.last_commit_position = r.read_int64(bytes)?,
                Ok(18) => msg.subscription_id = r.read_string(bytes).map(Cow::Borrowed)?,
                Ok(24) => msg.last_event_number = Some(r.read_int32(bytes)?),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for PersistentSubscriptionConfirmation<'a> {
    fn get_size(&self) -> usize {
        0
        + 1 + sizeof_varint(*(&self.last_commit_position) as u64)
        + 1 + sizeof_len((&self.subscription_id).len())
        + self.last_event_number.as_ref().map_or(0, |m| 1 + sizeof_varint(*(m) as u64))
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        w.write_with_tag(8, |w| w.write_int64(*&self.last_commit_position))?;
        w.write_with_tag(18, |w| w.write_string(&**&self.subscription_id))?;
        if let Some(ref s) = self.last_event_number { w.write_with_tag(24, |w| w.write_int32(*s))?; }
        Ok(())
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct PersistentSubscriptionStreamEventAppeared<'a> {
    pub event: ResolvedIndexedEvent<'a>,
}

impl<'a> PersistentSubscriptionStreamEventAppeared<'a> {
    pub fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.event = r.read_message(bytes, ResolvedIndexedEvent::from_reader)?,
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for PersistentSubscriptionStreamEventAppeared<'a> {
    fn get_size(&self) -> usize {
        0
        + 1 + sizeof_len((&self.event).get_size())
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        w.write_with_tag(10, |w| w.write_message(&self.event))?;
        Ok(())
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct SubscribeToStream<'a> {
    pub event_stream_id: Cow<'a, str>,
    pub resolve_link_tos: bool,
}

impl<'a> SubscribeToStream<'a> {
    pub fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.event_stream_id = r.read_string(bytes).map(Cow::Borrowed)?,
                Ok(16) => msg.resolve_link_tos = r.read_bool(bytes)?,
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for SubscribeToStream<'a> {
    fn get_size(&self) -> usize {
        0
        + 1 + sizeof_len((&self.event_stream_id).len())
        + 1 + sizeof_varint(*(&self.resolve_link_tos) as u64)
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        w.write_with_tag(10, |w| w.write_string(&**&self.event_stream_id))?;
        w.write_with_tag(16, |w| w.write_bool(*&self.resolve_link_tos))?;
        Ok(())
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct SubscriptionConfirmation {
    pub last_commit_position: i64,
    pub last_event_number: Option<i32>,
}

impl SubscriptionConfirmation {
    pub fn from_reader(r: &mut BytesReader, bytes: &[u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.last_commit_position = r.read_int64(bytes)?,
                Ok(16) => msg.last_event_number = Some(r.read_int32(bytes)?),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl MessageWrite for SubscriptionConfirmation {
    fn get_size(&self) -> usize {
        0
        + 1 + sizeof_varint(*(&self.last_commit_position) as u64)
        + self.last_event_number.as_ref().map_or(0, |m| 1 + sizeof_varint(*(m) as u64))
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        w.write_with_tag(8, |w| w.write_int64(*&self.last_commit_position))?;
        if let Some(ref s) = self.last_event_number { w.write_with_tag(16, |w| w.write_int32(*s))?; }
        Ok(())
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct StreamEventAppeared<'a> {
    pub event: ResolvedEvent<'a>,
}

impl<'a> StreamEventAppeared<'a> {
    pub fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.event = r.read_message(bytes, ResolvedEvent::from_reader)?,
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for StreamEventAppeared<'a> {
    fn get_size(&self) -> usize {
        0
        + 1 + sizeof_len((&self.event).get_size())
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        w.write_with_tag(10, |w| w.write_message(&self.event))?;
        Ok(())
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct UnsubscribeFromStream { }

impl UnsubscribeFromStream {
    pub fn from_reader(r: &mut BytesReader, _: &[u8]) -> Result<Self> {
        r.read_to_end();
        Ok(Self::default())
    }
}

impl MessageWrite for UnsubscribeFromStream { }

#[derive(Debug, Default, PartialEq, Clone)]
pub struct SubscriptionDropped {
    pub reason: mod_SubscriptionDropped::SubscriptionDropReason,
}

impl SubscriptionDropped {
    pub fn from_reader(r: &mut BytesReader, bytes: &[u8]) -> Result<Self> {
        let mut msg = SubscriptionDropped {
            reason: mod_SubscriptionDropped::SubscriptionDropReason::Unsubscribed,
            ..Self::default()
        };
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.reason = r.read_enum(bytes)?,
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl MessageWrite for SubscriptionDropped {
    fn get_size(&self) -> usize {
        0
        + if self.reason == mod_SubscriptionDropped::SubscriptionDropReason::Unsubscribed { 0 } else { 1 + sizeof_varint(*(&self.reason) as u64) }
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        if self.reason != mod_SubscriptionDropped::SubscriptionDropReason::Unsubscribed { w.write_with_tag(8, |w| w.write_enum(*&self.reason as i32))?; }
        Ok(())
    }
}

pub mod mod_SubscriptionDropped {


#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SubscriptionDropReason {
    Unsubscribed = 0,
    AccessDenied = 1,
    NotFound = 2,
    PersistentSubscriptionDeleted = 3,
    SubscriberMaxCountReached = 4,
}

impl Default for SubscriptionDropReason {
    fn default() -> Self {
        SubscriptionDropReason::Unsubscribed
    }
}

impl From<i32> for SubscriptionDropReason {
    fn from(i: i32) -> Self {
        match i {
            0 => SubscriptionDropReason::Unsubscribed,
            1 => SubscriptionDropReason::AccessDenied,
            2 => SubscriptionDropReason::NotFound,
            3 => SubscriptionDropReason::PersistentSubscriptionDeleted,
            4 => SubscriptionDropReason::SubscriberMaxCountReached,
            _ => Self::default(),
        }
    }
}

}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct NotHandled<'a> {
    pub reason: Option<mod_NotHandled::NotHandledReason>,
    pub additional_info: Option<Cow<'a, [u8]>>,
}

impl<'a> NotHandled<'a> {
    pub fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.reason = Some(r.read_enum(bytes)?),
                Ok(18) => msg.additional_info = Some(r.read_bytes(bytes).map(Cow::Borrowed)?),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for NotHandled<'a> {
    fn get_size(&self) -> usize {
        0
        + self.reason.as_ref().map_or(0, |m| 1 + sizeof_varint(*(m) as u64))
        + self.additional_info.as_ref().map_or(0, |m| 1 + sizeof_len((m).len()))
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        if let Some(ref s) = self.reason { w.write_with_tag(8, |w| w.write_enum(*s as i32))?; }
        if let Some(ref s) = self.additional_info { w.write_with_tag(18, |w| w.write_bytes(&**s))?; }
        Ok(())
    }
}

pub mod mod_NotHandled {

use std::borrow::Cow;
use super::*;

#[derive(Debug, Default, PartialEq, Clone)]
pub struct MasterInfo<'a> {
    pub external_tcp_address: Cow<'a, str>,
    pub external_tcp_port: i32,
    pub external_http_address: Cow<'a, str>,
    pub external_http_port: i32,
    pub external_secure_tcp_address: Option<Cow<'a, str>>,
    pub external_secure_tcp_port: Option<i32>,
}

impl<'a> MasterInfo<'a> {
    pub fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.external_tcp_address = r.read_string(bytes).map(Cow::Borrowed)?,
                Ok(16) => msg.external_tcp_port = r.read_int32(bytes)?,
                Ok(26) => msg.external_http_address = r.read_string(bytes).map(Cow::Borrowed)?,
                Ok(32) => msg.external_http_port = r.read_int32(bytes)?,
                Ok(42) => msg.external_secure_tcp_address = Some(r.read_string(bytes).map(Cow::Borrowed)?),
                Ok(48) => msg.external_secure_tcp_port = Some(r.read_int32(bytes)?),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for MasterInfo<'a> {
    fn get_size(&self) -> usize {
        0
        + 1 + sizeof_len((&self.external_tcp_address).len())
        + 1 + sizeof_varint(*(&self.external_tcp_port) as u64)
        + 1 + sizeof_len((&self.external_http_address).len())
        + 1 + sizeof_varint(*(&self.external_http_port) as u64)
        + self.external_secure_tcp_address.as_ref().map_or(0, |m| 1 + sizeof_len((m).len()))
        + self.external_secure_tcp_port.as_ref().map_or(0, |m| 1 + sizeof_varint(*(m) as u64))
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        w.write_with_tag(10, |w| w.write_string(&**&self.external_tcp_address))?;
        w.write_with_tag(16, |w| w.write_int32(*&self.external_tcp_port))?;
        w.write_with_tag(26, |w| w.write_string(&**&self.external_http_address))?;
        w.write_with_tag(32, |w| w.write_int32(*&self.external_http_port))?;
        if let Some(ref s) = self.external_secure_tcp_address { w.write_with_tag(42, |w| w.write_string(&**s))?; }
        if let Some(ref s) = self.external_secure_tcp_port { w.write_with_tag(48, |w| w.write_int32(*s))?; }
        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum NotHandledReason {
    NotReady = 0,
    TooBusy = 1,
    NotMaster = 2,
}

impl Default for NotHandledReason {
    fn default() -> Self {
        NotHandledReason::NotReady
    }
}

impl From<i32> for NotHandledReason {
    fn from(i: i32) -> Self {
        match i {
            0 => NotHandledReason::NotReady,
            1 => NotHandledReason::TooBusy,
            2 => NotHandledReason::NotMaster,
            _ => Self::default(),
        }
    }
}

}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct ScavengeDatabase { }

impl ScavengeDatabase {
    pub fn from_reader(r: &mut BytesReader, _: &[u8]) -> Result<Self> {
        r.read_to_end();
        Ok(Self::default())
    }
}

impl MessageWrite for ScavengeDatabase { }

#[derive(Debug, Default, PartialEq, Clone)]
pub struct ScavengeDatabaseCompleted<'a> {
    pub result: Option<mod_ScavengeDatabaseCompleted::ScavengeResult>,
    pub error: Option<Cow<'a, str>>,
    pub total_time_ms: i32,
    pub total_space_saved: i64,
}

impl<'a> ScavengeDatabaseCompleted<'a> {
    pub fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.result = Some(r.read_enum(bytes)?),
                Ok(18) => msg.error = Some(r.read_string(bytes).map(Cow::Borrowed)?),
                Ok(24) => msg.total_time_ms = r.read_int32(bytes)?,
                Ok(32) => msg.total_space_saved = r.read_int64(bytes)?,
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for ScavengeDatabaseCompleted<'a> {
    fn get_size(&self) -> usize {
        0
        + self.result.as_ref().map_or(0, |m| 1 + sizeof_varint(*(m) as u64))
        + self.error.as_ref().map_or(0, |m| 1 + sizeof_len((m).len()))
        + 1 + sizeof_varint(*(&self.total_time_ms) as u64)
        + 1 + sizeof_varint(*(&self.total_space_saved) as u64)
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        if let Some(ref s) = self.result { w.write_with_tag(8, |w| w.write_enum(*s as i32))?; }
        if let Some(ref s) = self.error { w.write_with_tag(18, |w| w.write_string(&**s))?; }
        w.write_with_tag(24, |w| w.write_int32(*&self.total_time_ms))?;
        w.write_with_tag(32, |w| w.write_int64(*&self.total_space_saved))?;
        Ok(())
    }
}

pub mod mod_ScavengeDatabaseCompleted {


#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ScavengeResult {
    Success = 0,
    InProgress = 1,
    Failed = 2,
}

impl Default for ScavengeResult {
    fn default() -> Self {
        ScavengeResult::Success
    }
}

impl From<i32> for ScavengeResult {
    fn from(i: i32) -> Self {
        match i {
            0 => ScavengeResult::Success,
            1 => ScavengeResult::InProgress,
            2 => ScavengeResult::Failed,
            _ => Self::default(),
        }
    }
}

}
