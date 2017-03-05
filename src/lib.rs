//! Tokio-based [EventStore](https://geteventstore.com/) client library in it's early stages.
//! Currently the most interesting API is the `tokio_service::service::Service` implemented by
//! `client::Client`, which allows sending values of `Package` to get back a `Future` of a response
//! `Package`. `Package` is the name for a frame in the protocol. See it's documentation for more
//! information.
//!
//! You can build values of `Package` using `builder::Builder` and it's functions. Actual payloads
//! are described as `Message` enum.
//!
//! The protocol is multiplexed so you can have multiple (hard limit is 128 currently) calls going
//! at any point in time. Current implementation is based on `tokio_proto::multiplex`, at the
//! moment using a custom fork. It does not yet support `tokio_proto::streaming::multiplex` which is
//! needed to support subscriptions.
//!
//! # Panics
//!
//!   * members of `builder` module can panic in a number of places (documented)
//!   * panics when decoding missing protobuf `required` values
//!
//! # Simplest example
//!
//! Example of writing to the database and simple handling of the response.
//!
//! ```no_run
//! extern crate futures;
//! extern crate tokio_core;
//! extern crate tokio_proto;
//! extern crate tokio_service;
//! extern crate eventstore_tcp;
//!
//! use std::net::SocketAddr;
//! use futures::Future;
//! use tokio_core::reactor::Core;
//! use tokio_service::Service;
//!
//! use eventstore_tcp::{EventStoreClient, Builder, Message, StreamVersion, ContentType};
//!
//! fn main() {
//!     let addr = "127.0.0.1:1113".parse::<SocketAddr>().unwrap();
//!     let mut core = Core::new().unwrap();
//!
//!     // connecting the client returns a future for an EventStoreClient
//!     // which implements tokio_service::Service
//!     let client = EventStoreClient::connect(&addr, &core.handle());
//!
//!     let value = client.and_then(|client| {
//!         // once the connection is made and EventStoreClient (`client`)
//!         // is created, send a WriteEvents request:
//!         client.call(Builder::write_events()
//!             .stream_id("my_stream-1")
//!             .expected_version(StreamVersion::from(42))
//!             .new_event()
//!                 .event_type("meaning_of_life")
//!                 .data("{ 'meaning': 42 }".as_bytes())
//!                 .data_content_type(ContentType::Json)
//!             .done()
//!             .build_package(None, None))
//!
//!         // call returns a future representing the response
//!     }).and_then(|resp| {
//!         match resp.message {
//!             Message::WriteEventsCompleted(Ok(_)) =>
//!                 println!("Event was written successfully"),
//!             Message::WriteEventsCompleted(Err(fail)) =>
//!                 println!("Event writing failed: {:?}", fail),
//!             unexpected => println!("Unexpected response: {:#?}", unexpected),
//!         };
//!
//!         Ok(())
//!     });
//!
//!     core.run(value).unwrap();
//! }
//! ```
//!
//! More examples can be found in the aspiring command line tool under `examples/testclient`.
#![deny(missing_docs)]

#[macro_use]
extern crate bitflags;
extern crate quick_protobuf;
extern crate uuid;
extern crate byteorder;
#[macro_use]
extern crate error_chain;
extern crate futures;
extern crate tokio_core;
extern crate tokio_proto;
extern crate tokio_service;
#[macro_use]
extern crate rental;

#[cfg(test)]
extern crate rustc_serialize;

use std::io;
use std::ops::Deref;
use std::borrow::Cow;
use tokio_core::io::EasyBuf;

mod client_messages;
pub use client_messages::{WriteEvents, ResolvedIndexedEvent, EventRecord, ReadAllEvents};
pub use client_messages::mod_NotHandled::{NotHandledReason, MasterInfo};

mod client_messages_ext;

mod write_events;
pub use write_events::{WriteEventsCompleted, WriteEventsFailure};

mod read_event;
pub use read_event::ReadEventFailure;

mod read_stream;
pub use read_stream::{ReadStreamSuccess, ReadStreamFailure};

mod read_all;
pub use read_all::{ReadAllSuccess, ReadAllFailure};

mod package;
pub use package::Package;

mod codec;

mod client;
pub use client::EventStoreClient;

pub mod builder;
pub use builder::Builder;

mod auth;
pub use auth::UsernamePassword;

mod raw;

mod errors {
    use std::str;
    use std::io;

    error_chain! {
        errors {
            InvalidFlags(flags: u8) {
                display("Invalid flags: 0x{:02x}", flags)
            }
            UnsupportedDiscriminator(d: u8) {
                display("Unsupported discriminator: 0x{:02x}", d)
            }
        }
    }

    impl Into<io::Error> for Error {
        fn into(self) -> io::Error {
            match self {
                e => io::Error::new(io::ErrorKind::Other, e),
            }
        }
    }

    impl Into<io::Error> for ErrorKind {
        fn into(self) -> io::Error {
            Error::from(self).into()
        }
    }
}

use self::errors::ErrorKind;

/// The direction in which events are read.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ReadDirection {
    /// Read from first (event 0) to the latest
    Forward,
    /// Read from latest (highest event number) to the first (event 0)
    Backward
}

impl Copy for ReadDirection {}

/// `ExpectedVersion` represents the different modes of optimistic locking when writing to a stream
/// using `WriteEventsBuilder`.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ExpectedVersion {
    /// No optimistic locking
    Any,
    /// Expect a stream not to exist
    NewStream,
    /// Expect exact number of events in the stream
    Exact(StreamVersion)
}

impl Copy for ExpectedVersion {}

impl Into<i32> for ExpectedVersion {
    /// Returns the wire representation.
    fn into(self) -> i32 {
        use self::ExpectedVersion::*;
        match self {
            Any => -2,
            NewStream => -1,
            Exact(ver) => ver.into()
        }
    }
}

impl From<StreamVersion> for ExpectedVersion {
    fn from(ver: StreamVersion) -> Self {
        ExpectedVersion::Exact(ver)
    }
}

/// `StreamVersion` represents the valid values for a stream version which is the same as the
/// event number of the latest event. As such, values are non-negative integers up to
/// `i32::max_value`. Negative values of `i32` have special meaning in the protocol, and are
/// restricted from used with this type.
///
/// Conversions to StreamVersion are quite horrible until TryFrom is stabilized.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct StreamVersion(u32);

impl Copy for StreamVersion {}

impl StreamVersion {
    /// Converts the value to a StreamVersion or panics if the value is out of range
    pub fn from(version: u32) -> Self {
        Self::from_opt(version).expect("StreamVersion overflow")
    }

    /// Converts the value to a StreamVersion returning None if the input is out of range.
    pub fn from_opt(version: u32) -> Option<Self> {
        // TODO: MAX_VALUE might be some magical value, should be lower?
        if version < i32::max_value() as u32 {
            Some(StreamVersion(version))
        } else {
            None
        }
    }

    #[doc(hidden)]
    pub fn from_i32(version: i32) -> Self {
        Self::from_i32_opt(version).expect("StreamVersion over/underflow")
    }

    #[doc(hidden)]
    pub fn from_i32_opt(version: i32) -> Option<Self> {
        if version < 0 {
            None
        } else if version == i32::max_value() {
            // this value is used to marking streams as deleted
            None
        } else {
            Some(StreamVersion(version as u32))
        }
    }
}

impl Into<i32> for StreamVersion {
    /// Returns the wire representation.
    fn into(self) -> i32 {
        self.0 as i32
    }
}

/// `EventNumber` is similar to `StreamVersion` and `ExpectedVersion` but is used when specifying a
/// position to read from in the stream. Allows specifying the first or last (when reading
/// backwards) event in addition to exact event number.
#[derive(Debug, Clone, Eq)]
pub enum EventNumber {
    /// The first event in a stream
    First,
    /// Exactly the given event number
    Exact(StreamVersion),
    /// The last event in a stream
    Last,
}

impl Copy for EventNumber {}

impl PartialEq<EventNumber> for EventNumber {
    fn eq(&self, other: &EventNumber) -> bool {
        let val: i32 = (*self).into();
        let other: i32 = (*other).into();
        val == other
    }
}

impl From<StreamVersion> for EventNumber {
    fn from(ver: StreamVersion) -> Self {
        EventNumber::Exact(ver)
    }
}

impl Into<i32> for EventNumber {
    /// Returns the wire representation.
    fn into(self) -> i32 {
        match self {
            EventNumber::First => 0,
            EventNumber::Exact(x) => x.into(),
            EventNumber::Last => -1
        }
    }
}

impl EventNumber {
    #[doc(hidden)]
    pub fn from_i32_opt(val: i32) -> Option<Self> {
        match val {
            0 => Some(EventNumber::First),
            -1 => Some(EventNumber::Last),
            x if x > 0 => Some(EventNumber::Exact(StreamVersion::from_i32(x))),
            invalid => {
                println!("invalid event number: {}", invalid);
                None
            }
        }
    }
}

/// Content type of the event `data` or `metadata`.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ContentType {
    /// Raw bytes
    Bytes,
    /// JSON values usable with projections in EventStore
    Json
}

impl Copy for ContentType {}

impl Into<i32> for ContentType {
    fn into(self) -> i32 {
        match self {
            ContentType::Bytes => 0,
            ContentType::Json => 1,
        }
    }
}

/// Enumeration of currently supported messages. The plan is to include every defined message while trying
/// to decode responses into `Result` alike messages, such as `ReadEventCompleted`.
#[derive(Debug, PartialEq)]
pub enum Message {
    /// Requests heartbeat from the other side. Unsure if clients or server sends these.
    HeartbeatRequest,
    /// Response to a heartbeat request.
    HeartbeatResponse,

    /// Ping request, similar to heartbeat.
    Ping,
    /// Ping response.
    Pong,

    /// Append to stream request
    WriteEvents(WriteEvents<'static>),
    /// Append to stream response, which can fail for a number of reasons
    WriteEventsCompleted(Result<WriteEventsCompleted, WriteEventsFailure>),

    /// Request to read a single event from a stream
    ReadEvent(client_messages::ReadEvent<'static>),
    /// Response to a single event read
    ReadEventCompleted(Result<client_messages::ResolvedIndexedEvent<'static>, ReadEventFailure>),

    /// Request to read a stream from a point forward or backward
    ReadStreamEvents(ReadDirection, client_messages::ReadStreamEvents<'static>),
    /// Response to a stream read in given direction
    ReadStreamEventsCompleted(ReadDirection, Result<ReadStreamSuccess, ReadStreamFailure>),

    /// Request to read a stream of all events from a position forward or backward
    ReadAllEvents(ReadDirection, client_messages::ReadAllEvents),
    /// Response to a read all in given direction
    ReadAllEventsCompleted(ReadDirection, Result<ReadAllSuccess, ReadAllFailure>),

    /// Request was not understood. Please open an issue!
    BadRequest(Option<String>),

    /// Correlated request was not handled. This is the likely response to requests where
    /// `require_master` is `true`, but the connected endpoint is not master and cannot reach it.
    NotHandled(NotHandledReason, Option<MasterInfo<'static>>),

    /// Request to authenticate attached credentials.
    Authenticate,

    /// Positive authentication response. The credentials used to `Authenticate` previously can be
    /// used in successive requests.
    Authenticated,

    /// Negative authentication response, or response to any sent request for which used
    /// authentication was not accepted.
    NotAuthenticated
}

/// Trait allows converting values to wire structs that borrow data from the implementing type.
/// Does not work as well as hoped if there is some data to borrow.
trait AsMessageWrite<M: quick_protobuf::MessageWrite> {
    fn as_message_write(&self) -> M;
}

/// Global unique position in the EventStore, used when reading all events.
/// Range -1..i64::max_value()
#[derive(Debug, Clone, Eq, PartialOrd, Ord)]
pub enum LogPosition {
    /// The first event ever
    First,
    /// Exact position
    Exact(u64),
    /// The last event written to the database at the moment
    Last,
}

impl Copy for LogPosition {}

impl PartialEq<LogPosition> for LogPosition {
    fn eq(&self, other: &LogPosition) -> bool {
        let left: i64 = (*self).into();
        let right: i64 = (*other).into();
        left == right
    }
}

impl From<i64> for LogPosition {
    fn from(val: i64) -> LogPosition {
        match LogPosition::from_i64_opt(val) {
            Some(x) => x,
            None => panic!("LogPosition undeflow: {}", val),
        }
    }
}

impl Into<i64> for LogPosition {
    fn into(self) -> i64 {
        match self {
            LogPosition::First => 0,
            LogPosition::Exact(x) => x as i64,
            LogPosition::Last => -1,
        }
    }
}

impl LogPosition {
    /// Wraps the value into LogPosition or None, if it is larger than i64
    pub fn from_opt(pos: u64) -> Option<LogPosition> {
        match pos {
            0 => Some(LogPosition::First),
            pos => {
                if pos < i64::max_value() as u64 {
                    Some(LogPosition::Exact(pos))
                } else {
                    None
                }
            }
        }
    }

    #[doc(hidden)]
    pub fn from_i64_opt(pos: i64) -> Option<LogPosition> {
        match pos {
            0 => Some(LogPosition::First),
            -1 => Some(LogPosition::Last),
            pos => {
                if pos > 0 {
                    Some(LogPosition::Exact(pos as u64))
                } else {
                    None
                }
            }
        }
    }
}

impl Message {
    fn decode(discriminator: u8, buf: &mut EasyBuf) -> io::Result<Message> {
        use client_messages_ext::MasterInfoExt;

        macro_rules! parse {
            ($x:ty, $buf:expr) => {
                {
                    let mut reader = ::quick_protobuf::reader::BytesReader::from_bytes($buf);
                    let res: Result<$x, io::Error> = <$x>::from_reader(&mut reader, $buf).map_err(|x| x.into());
                    assert!(reader.is_eof());
                    res
                }
            }
        }

        Ok(match discriminator {
            // these hold no data
            0x01 => Self::without_data(Message::HeartbeatRequest, buf),
            0x02 => Self::without_data(Message::HeartbeatResponse, buf),
            0x03 => Self::without_data(Message::Ping, buf),
            0x04 => Self::without_data(Message::Pong, buf),

            0x82 => parse!(client_messages::WriteEvents, buf.as_slice())?.into(),
            0x83 => parse!(client_messages::WriteEventsCompleted, buf.as_slice())?.into(),

            0xB0 => parse!(client_messages::ReadEvent, buf.as_slice())?.into(),
            0xB1 => parse!(client_messages::ReadEventCompleted, buf.as_slice())?.into(),

            0xB2 => (ReadDirection::Forward, parse!(client_messages::ReadStreamEvents, buf.as_slice())?).into(),
            0xB3 => (ReadDirection::Forward, parse!(client_messages::ReadStreamEventsCompleted, buf.as_slice())?).into(),
            0xB4 => (ReadDirection::Backward, parse!(client_messages::ReadStreamEvents, buf.as_slice())?).into(),
            0xB5 => (ReadDirection::Backward, parse!(client_messages::ReadStreamEventsCompleted, buf.as_slice())?).into(),

            0xB6 => (ReadDirection::Forward, parse!(client_messages::ReadAllEvents, buf.as_slice())?).into(),
            0xB7 => (ReadDirection::Forward, parse!(client_messages::ReadAllEventsCompleted, buf.as_slice())?).into(),
            0xB8 => (ReadDirection::Backward, parse!(client_messages::ReadAllEvents, buf.as_slice())?).into(),
            0xB9 => (ReadDirection::Backward, parse!(client_messages::ReadAllEventsCompleted, buf.as_slice())?).into(),

            0xF0 => {
                let info = if buf.len() > 0 {
                    Some(String::from_utf8_lossy(buf.as_slice()).into_owned())
                } else {
                    None
                };
                Message::BadRequest(info)
            },

            0xF1 => {
                let mut reason = parse!(client_messages::NotHandled, buf.as_slice())?;

                let master_info = reason.additional_info.take()
                    .map(|bytes| {
                         parse!(MasterInfo, bytes.deref())
                             .map(|x| x.into_owned())
                             .map(Option::Some)
                             .unwrap_or(None)
                    })
                    .and_then(|x| x);

                Message::NotHandled(reason.reason.unwrap(), master_info)
            },

            0xF2 => Self::without_data(Message::Authenticate, buf),

            0xF3 => Self::without_data(Message::Authenticated, buf),

            // server might send some reason here
            0xF4 => Self::without_data(Message::NotAuthenticated, buf),

            x => bail!(ErrorKind::UnsupportedDiscriminator(x)),
        })
    }

    fn without_data(ret: Message, _: &mut EasyBuf) -> Message {
        // Not sure what to do, currently just be lenient
        // let len = buf.len();
        //
        // if len != 0 {
        //     println!("Decoding {:?}: Discarding {} bytes of junk payload",
        //              ret,
        //              len);
        // }
        ret
    }

    fn encode<W: io::Write>(&self, w: &mut W) -> io::Result<()> {
        use Message::*;
        use quick_protobuf::MessageWrite;
        use client_messages_ext::ResolvedIndexedEventExt;

        macro_rules! encode {
            ($x: expr, $w: expr) => {
                {
                    let mut writer = quick_protobuf::writer::Writer::new($w);
                    let result: Result<(), io::Error> = $x.write_message(&mut writer)
                        .map_err(|x| x.into());
                    result
                }
            }
        }

        Ok(match *self {
            HeartbeatRequest |
            HeartbeatResponse |
            Ping |
            Pong |
            Authenticate |
            Authenticated |
            NotAuthenticated => (),

            WriteEvents(ref x) => encode!(x, w)?,

            WriteEventsCompleted(Ok(ref x)) => encode!(x.as_message_write(), w)?,
            WriteEventsCompleted(Err(ref x)) => encode!(x.as_message_write(), w)?,

            ReadEvent(ref re) => encode!(re, w)?,
            ReadEventCompleted(Ok(ref rie)) => encode!(rie.as_read_event_completed(), w)?,
            ReadEventCompleted(Err(ref fail)) => encode!(fail.as_message_write(), w)?,

            ReadStreamEvents(_, ref body) => encode!(body, w)?,
            ReadStreamEventsCompleted(_, Ok(ref success)) => encode!(success.as_message_write(), w)?,
            ReadStreamEventsCompleted(_, Err(ref why)) => encode!(why.as_message_write(), w)?,

            ReadAllEvents(_, ref body) => encode!(body, w)?,
            ReadAllEventsCompleted(_, Ok(ref success)) => encode!(success.as_message_write(), w)?,
            ReadAllEventsCompleted(_, Err(ref why)) => encode!(why.as_message_write(), w)?,

            BadRequest(Some(ref info)) => w.write_all(info.as_bytes())?,
            BadRequest(None) => (),
            NotHandled(ref reason, ref x) => {

                let additional_info = match x {
                    &Some(ref info) => {
                        let mut buf = Vec::new();
                        encode!(info, &mut buf)?;
                        Some(Cow::Owned(buf))
                    },
                    &None => None
                };

                let msg = client_messages::NotHandled {
                    reason: Some(*reason),
                    additional_info: additional_info,
                };

                encode!(msg, w)?
            },
        })
    }

    /// In the header of each Package there is single byte discriminator value for the type of the
    /// body.
    fn discriminator(&self) -> u8 {
        use Message::*;
        match *self {
            HeartbeatRequest => 0x01,
            HeartbeatResponse => 0x02,
            Ping => 0x03,
            Pong => 0x04,

            WriteEvents(_) => 0x82,
            WriteEventsCompleted(_) => 0x83,

            ReadEvent(_) => 0xB0,
            ReadEventCompleted(_) => 0xB1,

            ReadStreamEvents(ReadDirection::Forward, _) => 0xB2,
            ReadStreamEventsCompleted(ReadDirection::Forward, _) => 0xB3,

            ReadStreamEvents(ReadDirection::Backward, _) => 0xB4,
            ReadStreamEventsCompleted(ReadDirection::Backward, _) => 0xB5,

            ReadAllEvents(ReadDirection::Forward, _) => 0xB6,
            ReadAllEventsCompleted(ReadDirection::Forward, _) => 0xB7,

            ReadAllEvents(ReadDirection::Backward, _) => 0xB8,
            ReadAllEventsCompleted(ReadDirection::Backward, _) => 0xB9,

            BadRequest(_) => 0xf0,
            NotHandled(..) => 0xf1,
            Authenticate => 0xf2,
            Authenticated => 0xf3,
            NotAuthenticated => 0xf4
        }
    }
}
