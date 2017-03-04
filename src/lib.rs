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

#[cfg(test)]
extern crate rustc_serialize;

use std::io;
use std::ops::{Deref, Range};
use std::borrow::Cow;
use tokio_core::io::EasyBuf;

mod client_messages;
pub use client_messages::{WriteEvents, ResolvedIndexedEvent, ReadAllEvents};
use client_messages::ReadAllEventsCompleted;
pub use client_messages::mod_NotHandled::{NotHandledReason, MasterInfo};

mod client_messages_ext;

mod failures;
pub use failures::{WriteEventsFailure, ReadEventFailure, ReadStreamFailure, ReadAllFailure};

mod package;
pub use package::Package;

mod codec;

mod client;
pub use client::EventStoreClient;

pub mod builder;
pub use builder::{Builder, ExpectedVersion, StreamVersion, EventNumber, ContentType};

mod auth;
pub use auth::UsernamePassword;

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

/// Successful response to `Message::WriteEvents`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WriteEventsCompleted {
    /// The event number range assigned to the written events
    pub event_numbers: Range<i32>,

    /// Not public: missing type for positive i64
    prepare_position: Option<i64>,

    /// Not public: missing type for positive i64
    commit_position: Option<i64>,
}

/// Trait allows converting values to wire structs that borrow data from the implementing type.
/// Does not work as well as hoped if there is some data to borrow.
trait AsMessageWrite<M: quick_protobuf::MessageWrite> {
    fn as_message_write(&self) -> M;
}

impl AsMessageWrite<client_messages::WriteEventsCompleted<'static>> for WriteEventsCompleted {
    fn as_message_write(&self) -> client_messages::WriteEventsCompleted<'static> {
        client_messages::WriteEventsCompleted {
            result: Some(client_messages::OperationResult::Success),
            message: None,
            first_event_number: self.event_numbers.start,
            last_event_number: self.event_numbers.end - 1,
            prepare_position: self.prepare_position,
            commit_position: self.commit_position
        }
    }
}

impl<'a> From<(ReadDirection, client_messages::ReadStreamEvents<'a>)> for Message {
    fn from((dir, body): (ReadDirection, client_messages::ReadStreamEvents<'a>)) -> Message {
        use client_messages_ext::ReadStreamEventsExt;
        Message::ReadStreamEvents(dir, body.into_owned())
    }
}

/// Successful response to a `Message::ReadStreamEvents`.
#[derive(Debug, PartialEq, Clone)]
pub struct ReadStreamSuccess {
    /// The actual events returned by the server. Subject to `resolve_link_tos` setting on the read
    /// request.
    pub events: Vec<ResolvedIndexedEvent<'static>>,
    /// `EventNumber` for a query for the next page in the same direction, `None` if start has been
    /// reached when reading backwards. When reading forwards, this will never be `None` as new
    /// events might have appeared while receiving this response.
    pub next_page: Option<EventNumber>,
    /// Event number of the last event
    pub last_event_number: StreamVersion,
    /// Has the end of the stream been reached (or could more events be read immediatedly)
    pub end_of_stream: bool,

    /// Last commit position of the last event. Not public as there is currently no type for an
    /// positive i64 (0 < x < i64). Also, not sure how to explain the use of this property.
    last_commit_position: i64,
}

impl<'a> From<(ReadDirection, client_messages::ReadStreamEventsCompleted<'a>)> for Message {
    fn from((dir, completed): (ReadDirection, client_messages::ReadStreamEventsCompleted<'a>)) -> Message {
        use client_messages::mod_ReadStreamEventsCompleted::ReadStreamResult;
        use client_messages_ext::ResolvedIndexedEventExt;

        match completed.result {
            Some(ReadStreamResult::Success) => {
                let next_page = if dir == ReadDirection::Backward && completed.next_event_number < 0 {
                    None
                } else {
                    EventNumber::from_i32_opt(completed.next_event_number)
                };

                // TODO: from_i32 can still panic but haven't found a legitimate situation

                Message::ReadStreamEventsCompleted(dir, Ok(ReadStreamSuccess {
                    events: completed.events.into_iter().map(|x| x.into_owned()).collect(),
                    next_page: next_page,
                    last_event_number: StreamVersion::from_i32(completed.last_event_number),
                    end_of_stream: completed.is_end_of_stream,
                    last_commit_position: completed.last_commit_position,
                }))
            },

            Some(err) => {
                // TODO: last_commit_position has readable value which is discarded here
                Message::ReadStreamEventsCompleted(dir, Err((err, completed.error).into()))
            },

            None => panic!("No result found from ReadStreamEventsCompleted"),
        }
    }
}

impl ReadStreamSuccess {
    #[doc(hidden)]
    pub fn as_message_write<'a>(&'a self) -> client_messages::ReadStreamEventsCompleted<'a> {
        use client_messages::mod_ReadStreamEventsCompleted::ReadStreamResult;
        use client_messages_ext::ResolvedIndexedEventExt;

        client_messages::ReadStreamEventsCompleted {
            events: self.events.iter().map(|x| x.borrowed()).collect(),
            result: Some(ReadStreamResult::Success),
            next_event_number: self.next_page.map(|x| x.into()).unwrap_or(-1),
            last_event_number: self.last_event_number.into(),
            is_end_of_stream: self.end_of_stream,
            last_commit_position: self.last_commit_position,
            error: None,
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

impl From<(ReadDirection, ReadAllEvents)> for Message {
    fn from((dir, rae): (ReadDirection, ReadAllEvents)) -> Message {
        Message::ReadAllEvents(dir, rae)
    }
}

/// Successful response to `Message::ReadAllEvents`.
#[derive(Debug, Clone, PartialEq)]
pub struct ReadAllSuccess {
    commit_position: LogPosition,
    prepare_position: LogPosition,
    /// The read events, with position metadata
    pub events: Vec<ResolvedEvent<'static>>,
    next_commit_position: Option<LogPosition>,
    next_prepare_position: Option<LogPosition>,
}

/// Read event in `ReadAllSuccess` response
#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedEvent<'a> {
    /// The read event
    pub event: client_messages::EventRecord<'a>,
    /// Possible linking event
    pub link: Option<client_messages::EventRecord<'a>>,
    /// Position where this events transaction is commited
    pub commit_position: LogPosition,
    /// Position where this event is stored
    pub prepare_position: LogPosition,
}

impl<'a> From<client_messages::ResolvedEvent<'a>> for ResolvedEvent<'a> {
    fn from(e: client_messages::ResolvedEvent<'a>) -> ResolvedEvent<'a> {
        ResolvedEvent {
            event: e.event.into(),
            link: e.link.into(),
            commit_position: e.commit_position.into(),
            prepare_position: e.prepare_position.into()
        }
    }
}

impl<'a> ResolvedEvent<'a> {
    fn into_owned(self) -> ResolvedEvent<'static> {
        use client_messages_ext::EventRecordExt;
        ResolvedEvent {
            event: self.event.into_owned(),
            link: self.link.map(|link| link.into_owned()),
            commit_position: self.commit_position,
            prepare_position: self.prepare_position,
        }
    }

    fn borrowed<'b>(&'b self) -> client_messages::ResolvedEvent<'b> {
        use client_messages_ext::EventRecordExt;
        client_messages::ResolvedEvent {
            event: self.event.borrowed(),
            link: self.link.as_ref().map(|x| x.borrowed()),
            commit_position: self.commit_position.into(),
            prepare_position: self.prepare_position.into(),
        }
    }
}

impl<'a> From<(ReadDirection, ReadAllEventsCompleted<'a>)> for Message {
    fn from((dir, resp): (ReadDirection, ReadAllEventsCompleted<'a>)) -> Message {
        use client_messages::mod_ReadAllEventsCompleted::ReadAllResult;

        let res = match resp.result {
            ReadAllResult::Success => {
                Ok(ReadAllSuccess {
                    commit_position: resp.commit_position.into(),
                    prepare_position: resp.prepare_position.into(),
                    events: resp.events.into_iter().map(|x| {
                        let er: ResolvedEvent<'a> = x.into();
                        er.into_owned()
                    }).collect(),
                    next_commit_position: LogPosition::from_i64_opt(resp.next_commit_position),
                    next_prepare_position: LogPosition::from_i64_opt(resp.next_prepare_position),
                })
            },
            fail => Err((fail, resp.error).into()),
        };

        Message::ReadAllEventsCompleted(dir, res)
    }
}

impl ReadAllSuccess {
    fn as_message_write<'a>(&'a self) -> client_messages::ReadAllEventsCompleted<'a> {
        use client_messages::mod_ReadAllEventsCompleted::ReadAllResult;

        client_messages::ReadAllEventsCompleted {
            commit_position: self.commit_position.into(),
            prepare_position: self.prepare_position.into(),
            events: self.events.iter().map(|x| x.borrowed()).collect(),
            next_commit_position: self.next_commit_position.map(|x| x.into()).unwrap_or(-1),
            next_prepare_position: self.next_prepare_position.map(|x| x.into()).unwrap_or(-1),
            result: ReadAllResult::Success,
            error: None,
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

impl<'a> From<WriteEvents<'a>> for Message {
    fn from(we: WriteEvents<'a>) -> Self {
        use client_messages_ext::WriteEventsExt;
        Message::WriteEvents(we.into_owned())
    }
}

impl<'a> From<client_messages::ReadEvent<'a>> for Message {
    fn from(re: client_messages::ReadEvent<'a>) -> Self {
        use client_messages_ext::ReadEventExt;
        Message::ReadEvent(re.into_owned())
    }
}

// these two are problematic and would probably be best served by using tryfrom
// but as it's not yet stable, just use From with panics.

impl<'a> From<client_messages::WriteEventsCompleted<'a>> for Message {
    fn from(wec: client_messages::WriteEventsCompleted<'a>) -> Self {
        use client_messages::OperationResult::*;

        // FIXME: can panic
        let res = match wec.result {
            Some(Success) => {
                Ok(WriteEventsCompleted {
                    // off-by one: Range is [start, end)
                    event_numbers: wec.first_event_number..wec.last_event_number + 1,
                    prepare_position: wec.prepare_position,
                    commit_position: wec.commit_position,
                })
            }
            Some(other) => Err(other.into()),
            None => panic!("OperationResult was not found in the received message"),
        };

        Message::WriteEventsCompleted(res)
    }
}

impl<'a> From<client_messages::ReadEventCompleted<'a>> for Message {
    fn from(rec: client_messages::ReadEventCompleted<'a>) -> Self {
        use client_messages::mod_ReadEventCompleted::ReadEventResult;
        use client_messages_ext::ResolvedIndexedEventExt;

        // FIXME: can panic
        let res = match rec.result {
            Some(ReadEventResult::Success) => Ok(rec.event.into_owned()),
            Some(other) => Err((other, rec.error).into()),
            None => panic!("ReadEventResult was not found in the received message"),
        };

        Message::ReadEventCompleted(res)
    }
}
