//#![feature(plugin)]
// #![plugin(protobuf_macros)]

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

use std::fmt;
use std::io;
use std::ops::{Deref, Range};
use std::borrow::Cow;
use std::error::Error;
use byteorder::{ReadBytesExt, WriteBytesExt};
use tokio_core::io::EasyBuf;

mod messages;
use messages::mod_EventStore::mod_Client::mod_Messages as client_messages;
use messages::mod_EventStore::mod_Client::mod_Messages::WriteEvents;
use messages::mod_EventStore::mod_Client::mod_Messages::mod_NotHandled::{NotHandledReason, MasterInfo};

mod messages_ext;
use messages_ext::{WriteEventsExt, WriteEventsCompletedExt, MasterInfoExt};

mod failures;
pub use failures::{OperationFailure, ReadEventFailure};

mod package;
pub use package::Package;

mod codec;

mod client;
pub use client::EventStoreClient;

pub mod builder;
pub use builder::{Builder, ExpectedVersion, StreamVersion};

pub mod errors {
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

/// TODO: investigate if we could at least encode Cow<'a, str>
#[derive(Clone, PartialEq, Eq)]
pub struct UsernamePassword(Cow<'static, str>, Cow<'static, str>);

impl fmt::Debug for UsernamePassword {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "({:?}, PASSWORD)", self.0)
    }
}

impl UsernamePassword {
    pub fn new<S: Into<Cow<'static, str>>>(username: S, password: S) -> UsernamePassword {
        UsernamePassword(username.into(), password.into())
    }

    fn decode<R: ReadBytesExt>(buf: &mut R) -> io::Result<Self> {
        use std::string;

        fn convert_utf8_err(e: string::FromUtf8Error) -> io::Error {
            io::Error::new(io::ErrorKind::InvalidData, e.utf8_error())
        }

        let len = buf.read_u8()?;
        let mut username = vec![0u8, len];
        buf.read_exact(&mut username[..])?;
        let username = String::from_utf8(username).map_err(convert_utf8_err)?;

        let len = buf.read_u8()?;
        let mut password = vec![0u8, len];
        buf.read_exact(&mut password[..])?;
        let password = String::from_utf8(password).map_err(convert_utf8_err)?;

        Ok(UsernamePassword(Cow::Owned(username), Cow::Owned(password)))
    }

    fn encode<W: WriteBytesExt>(&self, buf: &mut W) -> io::Result<usize> {
        // TODO: new that disallows too long strings
        buf.write_u8(self.0.len() as u8)?;
        buf.write_all(self.0.as_bytes())?;
        buf.write_u8(self.1.len() as u8)?;
        buf.write_all(self.1.as_bytes())?;

        Ok(1 + self.0.len() + 1 + self.1.len())
    }
}

#[derive(Debug, PartialEq)]
pub enum Message {
    HeartbeatRequest,
    HeartbeatResponse,
    Ping,
    Pong,

    WriteEvents(WriteEvents<'static>),
    WriteEventsCompleted(Result<WriteEventsCompleted, Explanation>),

    /// Request was not understood
    BadRequest(Option<String>),

    /// Correlated request was not handled
    NotHandled(NotHandledReason, Option<MasterInfo<'static>>),

    /// Request to authenticate
    Authenticate,

    /// Positive authentication response
    Authenticated,

    /// Negative authentication response
    NotAuthenticated
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WriteEventsCompleted {
    pub event_numbers: Range<i32>,
    pub prepare_position: Option<i64>,
    pub commit_position: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Explanation {
    reason: OperationFailure,
    message: Option<String>
}

impl Explanation {
    fn new(or: client_messages::OperationResult, msg: Option<String>) -> Explanation {
        Explanation { reason: or.into(), message: msg }
    }
}

impl fmt::Display for Explanation {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self.message {
            Some(ref s) => write!(fmt, "{}: {}", self.description(), s),
            None => write!(fmt, "{}", self.description())
        }
    }
}

impl Error for Explanation {
    fn description(&self) -> &str {
        use OperationFailure::*;
        match self.reason {
            PrepareTimeout => "Internal server timeout, should be retried",
            CommitTimeout => "Internal server timeout, should be retried",
            ForwardTimeout => "Server timed out while awaiting response to forwarded request, should be retried",
            WrongExpectedVersion => "Stream version was not expected, optimistic locking failure",
            StreamDeleted => "Stream had been deleted",
            InvalidTransaction => "Transaction had been rolled back",
            AccessDenied => "Access to stream was denied"
        }
    }
}

trait AsMessageWrite<M: quick_protobuf::MessageWrite> {
    fn as_message_write(&self) -> M;

    fn encode<W: io::Write>(&self, out: &mut W) -> io::Result<()> {
        let conv = self.as_message_write();
        conv.write_message(&mut quick_protobuf::writer::Writer::new(out)).map_err(convert_qp_err)
    }
}

impl Explanation {
    fn as_write_events_completed<'a>(&'a self) -> client_messages::WriteEventsCompleted<'a> {
        client_messages::WriteEventsCompleted {
            result: Some(self.reason.into()),
            message: self.message.as_ref().map(|s| Cow::Borrowed(s.as_str())),
            first_event_number: -1,
            last_event_number: -1,
            prepare_position: None,
            commit_position: None,
        }
    }
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

fn convert_qp_err(e: ::quick_protobuf::errors::Error) -> io::Error {
    use std::io::{Error as IoError, ErrorKind as IoErrorKind};
    use quick_protobuf::errors::{Error, ErrorKind};

    match e {
        Error(ErrorKind::Io(e), _) => e,
        Error(ErrorKind::Utf8(e), _) => IoError::new(IoErrorKind::InvalidData, e.utf8_error()),
        Error(ErrorKind::StrUtf8(e), _) => IoError::new(IoErrorKind::InvalidData, e),
        x => IoError::new(IoErrorKind::Other, x)
    }
}

impl Message {
    fn decode(discriminator: u8, buf: &mut EasyBuf) -> io::Result<Message> {
        macro_rules! parse {
            ($x:ty, $buf:expr) => {
                {
                    let mut reader = ::quick_protobuf::reader::BytesReader::from_bytes($buf);
                    let res = <$x>::from_reader(&mut reader, $buf).map_err(convert_qp_err);
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

            /*
            0xB0 => { /* readevent */ }
            0xB1 => { /* readeventcompleted */ }
            0xB2 => { /* readstrmeventsfwd */ }
            0xB3 => { /* readstrmeventsfwdcompleted */ }
            0xB4 => { /* readstrmeventsbackwrd */ }
            0xB5 => { /* readstrmeventsbackwrdcompleted */ }
            0xB6 => { /* readalleventsfwd */ }
            0xB7 => { /* readalleventsfwdcompleted */ }
            0xB8 => { /* readalleventsbackwrd */ }
            0xB9 => { /* readalleventsbackwrdcompleted */ }
            */

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

        macro_rules! encode {
            ($x: expr, $w: ident) => {
                $x.write_message(&mut quick_protobuf::writer::Writer::new($w)).map_err(convert_qp_err)
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
            WriteEventsCompleted(Err(ref x)) => encode!(x.as_write_events_completed(), w)?,

            BadRequest(Some(ref info)) => w.write_all(info.as_bytes())?,
            BadRequest(None) => (),
            NotHandled(ref reason, ref x) => {

                let additional_info = match x {
                    &Some(ref info) => {
                        let mut buf = Vec::new();
                        info.write_message(&mut quick_protobuf::writer::Writer::new(&mut buf))
                            .map(move |_| buf)
                            .map(Cow::Owned)
                            .map(Option::Some)
                            .map_err(convert_qp_err)?
                    },
                    &None => None
                };

                let msg = client_messages::NotHandled {
                    reason: Some(*reason),
                    additional_info: additional_info,
                };

                msg.write_message(&mut quick_protobuf::writer::Writer::new(w)).map_err(convert_qp_err)?
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

            BadRequest(_) => 0xf0,
            NotHandled(..) => 0xf1,
            Authenticate => 0xf2,
            Authenticated => 0xf3,
            NotAuthenticated => 0xf4
        }
    }
}

impl<'a> From<WriteEvents<'a>> for Message {
    fn from(we: WriteEvents<'a>) -> Message {
        use messages_ext::WriteEventsExt;
        Message::WriteEvents(we.into_owned())
    }
}

impl<'a> From<client_messages::WriteEventsCompleted<'a>> for Message {
    fn from(wec: client_messages::WriteEventsCompleted<'a>) -> Message {
        use messages_ext::WriteEventsCompletedExt;
        wec.into_message()
    }
}
