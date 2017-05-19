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
//! #![feature(try_from)]
//!
//! extern crate futures;
//! extern crate tokio_core;
//! extern crate tokio_proto;
//! extern crate tokio_service;
//! extern crate eventstore_tcp;
//!
//! use std::convert::TryFrom;
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
//!             .expected_version(StreamVersion::try_from(42).unwrap()) // don't do that ?
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
//! More examples can be found in the aspiring command line tool under `testclient/`.
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
extern crate hex;

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

mod event_number;
pub use event_number::EventNumber;

mod stream_version;
pub use stream_version::StreamVersion;

mod expected_version;
pub use expected_version::ExpectedVersion;

mod log_position;
pub use log_position::LogPosition;

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
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            use self::ResultStatusKind::*;
            f.write_str(match *self {
                WriteEvents => "WriteEventsCompleted::result",
                ReadEvent => "ReadEventCompleted::result",
                ReadStream => "ReadStreamEventsCompleted::result",
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
                display("Invalid stream version: {}", value)
            }
            InvalidEventNumber(value: i32) {
                display("Invalid event number: {}", value)
            }
            UnderflowLogPosition(value: i64) {
                display("Invalid log position: {}", value)
            }
            OverflowLogPosition(value: u64) {
                display("Invalid log position: {}", value)
            }
            UnsupportedDiscriminator(d: u8) {
                display("Unsupported discriminator 0x{:02x}", d)
            }
            UnimplementedConversion {
                display("Unimplemented conversion")
            }
            WriteEventsInvalidTransaction {
                display("Unexpected write events result: invalid transaction")
            }
        }
    }

    impl Into<io::Error> for Error {
        fn into(self) -> io::Error {
            io::Error::new(io::ErrorKind::Other, self)
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
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ReadDirection {
    /// Read from first (event 0) to the latest
    Forward,
    /// Read from latest (highest event number) to the first (event 0)
    Backward
}

/// Content type of the event `data` or `metadata`.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ContentType {
    /// Raw bytes
    Bytes,
    /// JSON values usable with projections in EventStore
    Json
}

impl From<ContentType> for i32 {
    fn from(ctype: ContentType) -> Self {
        match ctype {
            ContentType::Bytes => 0,
            ContentType::Json => 1,
        }
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
