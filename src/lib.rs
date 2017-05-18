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
//! There should not be any panicing now that `adapted` and `raw` are separate.
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
//! use eventstore_tcp::{EventStoreClient, Builder, AdaptedMessage, StreamVersion, ContentType};
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
//!         // By default, `resp` is a `Package` that contains the raw protobuf defined message
//!         // (`RawMessage`). It is possible to refine it into AdaptedMessage which can fail:
//!         match resp.message.try_adapt().unwrap() {
//!             AdaptedMessage::WriteEventsCompleted(Ok(_)) =>
//!                 println!("Event was written successfully"),
//!             AdaptedMessage::WriteEventsCompleted(Err(fail)) =>
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
#![feature(try_from)]

#[macro_use]
extern crate bitflags;
extern crate quick_protobuf;
extern crate uuid;
extern crate byteorder;
#[macro_use]
extern crate error_chain;
extern crate futures;
extern crate tokio_io;
extern crate tokio_core;
extern crate tokio_proto;
extern crate tokio_service;
extern crate bytes;
#[macro_use]
extern crate derive_more;

#[cfg(test)]
extern crate rustc_serialize;

use std::str;
use std::convert::TryFrom;

pub mod raw;
pub use raw::RawMessage;
pub use raw::client_messages::{WriteEvents, ResolvedIndexedEvent, EventRecord, ReadAllEvents};
pub use raw::client_messages::mod_NotHandled::{NotHandledReason, MasterInfo};

pub mod adapted;
pub use adapted::AdaptedMessage;

pub mod package;
pub use package::Package;

pub mod codec;

mod client;
pub use client::EventStoreClient;

pub mod builder;
pub use builder::Builder;

mod auth;
pub use auth::UsernamePassword;

mod stream_version;
pub use stream_version::StreamVersion;

mod errors {
    use std::str;
    use std::io;
    use std::fmt;

    /// Enum describing the locations where a result value can be missing
    #[derive(Debug, PartialEq)]
    pub enum ResultStatusKind {
        /// Missing from WriteEventsCompleted
        WriteEvents,
        /// Missing from ReadEventCompleted
        ReadEvent,
        /// Missing from ReadStreamEventsCompleted
        ReadStream,
    }

    impl fmt::Display for ResultStatusKind {
        fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
            use self::ResultStatusKind::*;
            write!(fmt, "{}", match self {
                &WriteEvents => "WriteEventsCompleted::result",
                &ReadEvent => "ReadEventCompleted::result",
                &ReadStream => "ReadStreamEventsCompleted::result",
            })
        }
    }

    error_chain! {
        foreign_links {
            InvalidUtf8(str::Utf8Error);
        }

        errors {
            InvalidFlags(flags: u8) {
                display("Invalid flags: 0x{:02x}", flags)
            }
            MissingResultField(which: ResultStatusKind) {
                display("Missing result field: {}", which)
            }
            InvalidStreamVersion(value: i32) {
                display("Invalid StreamVersion: {}", value)
            }
            InvalidEventNumber(value: i32) {
                display("Invalid EventNumber: {}", value)
            }
            InvalidLogPosition(value: i64) {
                display("Invalid LogPosition: {}", value)
            }
            UnsupportedDiscriminator(d: u8) {
                display("Unsupported discriminator 0x{:02x}", d)
            }
            UnimplementedConversion {
                display("Unimplemented conversion")
            }
            WriteEventsInvalidTransaction {
                display("Unexpected WriteEvents result: InvalidTransaction")
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

use self::errors::{Error, ErrorKind};

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

impl CustomTryFrom<i32> for ExpectedVersion {
    type Err = Error;

    fn try_from(val: i32) -> Result<ExpectedVersion, (i32, Self::Err)> {
        let ver = StreamVersion::try_from(val)?;
        Ok(ExpectedVersion::from(ver))
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

impl CustomTryFrom<i32> for EventNumber {
    type Err = Error;

    fn try_from(val: i32) -> Result<Self, (i32, Self::Err)> {
        match val {
            0 => Ok(EventNumber::First),
            -1 => Ok(EventNumber::Last),
            x if x > 0 => Ok(EventNumber::Exact(StreamVersion::from_i32(x))),
            invalid => {
                Err((invalid, ErrorKind::InvalidEventNumber(val).into()))
            }
        }
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

impl CustomTryFrom<i64> for LogPosition {
    type Err = Error;

    fn try_from(val: i64) -> Result<Self, (i64, Self::Err)> {
        match val {
            0 => Ok(LogPosition::First),
            -1 => Ok(LogPosition::Last),
            pos => {
                if pos > 0 {
                    Ok(LogPosition::Exact(pos as u64))
                } else {
                    Err((pos, ErrorKind::InvalidLogPosition(pos).into()))
                }
            }
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
        Self::try_from(pos).ok()
    }
}

trait CustomTryFrom<T: Sized>: Sized {
    type Err;

    fn try_from(t: T) -> Result<Self, (T, Self::Err)>;
}

trait CustomTryInto<T: Sized>: Sized {
    type Err;

    fn try_into(self) -> Result<T, (Self, Self::Err)>;
}

impl<T, U> CustomTryInto<U> for T where U: CustomTryFrom<T> {
    type Err = U::Err;

    fn try_into(self) -> Result<U, (T, Self::Err)> {
        U::try_from(self)
    }
}
