#![allow(dead_code)]

use std::io;
use std::str;
use std::borrow::Cow;
use quick_protobuf;

use client_messages::{WriteEvents, WriteEventsCompleted, ReadEvent, ReadEventCompleted, ReadStreamEvents, ReadStreamEventsCompleted, ReadAllEvents, ReadAllEventsCompleted, NotHandled};

use {MappingError, ReadDirection};

#[derive(Debug, PartialEq, Clone)]
pub enum RawMessage<'a> {
    /// Requests heartbeat from the other side. Unsure if clients or server sends these.
    HeartbeatRequest,
    /// Response to a heartbeat request.
    HeartbeatResponse,

    /// Ping request, similar to heartbeat.
    Ping,
    /// Ping response.
    Pong,

    /// Append to stream request
    WriteEvents(WriteEvents<'a>),
    /// Append to stream response, which can fail for a number of reasons
    WriteEventsCompleted(WriteEventsCompleted<'a>),

    /// Request to read a single event from a stream
    ReadEvent(ReadEvent<'a>),
    /// Response to a single event read
    ReadEventCompleted(ReadEventCompleted<'a>),

    /// Request to read a stream from a point forward or backward
    ReadStreamEvents(ReadDirection, ReadStreamEvents<'a>),
    /// Response to a stream read in given direction
    ReadStreamEventsCompleted(ReadDirection, ReadStreamEventsCompleted<'a>),

    /// Request to read a stream of all events from a position forward or backward
    ReadAllEvents(ReadDirection, ReadAllEvents),
    /// Response to a read all in given direction
    ReadAllEventsCompleted(ReadDirection, ReadAllEventsCompleted<'a>),

    /// Request was not understood. Please open an issue!
    BadRequest(BadRequestPayload<'a>),

    /// Correlated request was not handled. This is the likely response to requests where
    /// `require_master` is `true`, but the connected endpoint is not master and cannot reach it.
    NotHandled(NotHandled<'a>),

    /// Request to authenticate attached credentials.
    Authenticate,

    /// Positive authentication response. The credentials used to `Authenticate` previously can be
    /// used in successive requests.
    Authenticated,

    /// Negative authentication response, or response to any sent request for which used
    /// authentication was not accepted. May contain a reason.
    NotAuthenticated(NotAuthenticatedPayload<'a>),

    /// Placeholder for a discriminator and the undecoded bytes
    Unsupported(u8, Cow<'a, [u8]>),
}

/// Trait for facilitating fallible Cow<'a, [u8]> -> Cow<'a, str> conversion.
pub trait ByteWrapper<'a>: Into<Cow<'a, [u8]>> + From<Cow<'a, [u8]>> {
    type ConversionErr: From<str::Utf8Error>;

    fn into_str_wrapper(self) -> Result<Cow<'a, str>, (Self, Self::ConversionErr)> {
        let plain: Cow<'a, [u8]> = self.into();
        match plain {
            Cow::Owned(vec) =>
                String::from_utf8(vec)
                    .map(|s| Cow::Owned(s))
                    .map_err(|e| {
                        let narrowed = e.utf8_error();
                        let revived = Self::from(Cow::Owned(e.into_bytes()));

                        (revived, narrowed.into())
                    }),
            Cow::Borrowed(buf) =>
                str::from_utf8(buf)
                    .map(|s| Cow::Borrowed(s))
                    .map_err(|e| (Self::from(Cow::Borrowed(buf)), e.into()))
        }
    }
}

/// Newtype for an arbitary NotAuthenticated "info", which could be
/// UTF8 string.
#[derive(Debug, PartialEq, Clone)]
pub struct NotAuthenticatedPayload<'a>(Cow<'a, [u8]>);

impl<'a> AsRef<[u8]> for NotAuthenticatedPayload<'a> {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl<'a> ByteWrapper<'a> for NotAuthenticatedPayload<'a> {
    type ConversionErr = MappingError;
}

impl<'a> From<Cow<'a, [u8]>> for NotAuthenticatedPayload<'a> {
    fn from(data: Cow<'a, [u8]>) -> NotAuthenticatedPayload<'a> {
        NotAuthenticatedPayload(data)
    }
}

impl<'a> Into<Cow<'a, [u8]>> for NotAuthenticatedPayload<'a> {
    fn into(self) -> Cow<'a, [u8]> {
        self.0
    }
}

/// Newtype for an arbitary BadRequest "info", which could be
/// UTF8 string.
#[derive(Debug, PartialEq, Clone)]
pub struct BadRequestPayload<'a>(Cow<'a, [u8]>);

impl<'a> AsRef<[u8]> for BadRequestPayload<'a> {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl<'a> ByteWrapper<'a> for BadRequestPayload<'a> {
    type ConversionErr = MappingError;
}

impl<'a> From<Cow<'a, [u8]>> for BadRequestPayload<'a> {
    fn from(data: Cow<'a, [u8]>) -> BadRequestPayload<'a> {
        BadRequestPayload(data)
    }
}

impl<'a> Into<Cow<'a, [u8]>> for BadRequestPayload<'a> {
    fn into(self) -> Cow<'a, [u8]> {
        self.0
    }
}

macro_rules! wrapup {
    ($x:ty, $f:expr) => {
        impl<'a> From<$x> for RawMessage<'a> {
            fn from(x: $x) -> RawMessage<'a> {
                $f(x)
            }
        }
    };
}

macro_rules! wrapup_directed {
    ($x: ty, $f: expr) => {
        impl<'a> From<(ReadDirection, $x)> for RawMessage<'a> {
            fn from((x, y): (ReadDirection, $x)) -> RawMessage<'a> {
                $f(x, y)
            }
        }
    };
}

wrapup!(BadRequestPayload<'a>, RawMessage::BadRequest);
wrapup!(NotAuthenticatedPayload<'a>, RawMessage::NotAuthenticated);
wrapup!(WriteEvents<'a>, RawMessage::WriteEvents);
wrapup!(WriteEventsCompleted<'a>, RawMessage::WriteEventsCompleted);
wrapup!(ReadEvent<'a>, RawMessage::ReadEvent);
wrapup!(ReadEventCompleted<'a>, RawMessage::ReadEventCompleted);
wrapup!(NotHandled<'a>, RawMessage::NotHandled);
wrapup_directed!(ReadStreamEvents<'a>, RawMessage::ReadStreamEvents);
wrapup_directed!(ReadStreamEventsCompleted<'a>, RawMessage::ReadStreamEventsCompleted);
wrapup_directed!(ReadAllEvents, RawMessage::ReadAllEvents);
wrapup_directed!(ReadAllEventsCompleted<'a>, RawMessage::ReadAllEventsCompleted);

impl<'a> From<(u8, Cow<'a, [u8]>)> for RawMessage<'a> {
    fn from((d, data): (u8, Cow<'a, [u8]>)) -> RawMessage<'a> {
        RawMessage::Unsupported(d, data)
    }
}

impl<'a> RawMessage<'a> {

    pub fn into_owned(self) -> RawMessage<'static> {
        unimplemented!()
    }

    /// Decodes the message from the buffer without any cloning.
    pub fn decode(discriminator: u8, buf: &'a [u8]) -> io::Result<RawMessage<'a>> {
        use self::RawMessage;
        use ReadDirection::{Forward, Backward};

        macro_rules! decode {
            ($x:ty, $buf:expr) => {
                {
                    let mut reader = ::quick_protobuf::reader::BytesReader::from_bytes($buf);
                    let res: Result<$x, io::Error> = <$x>::from_reader(&mut reader, $buf)
                        .map_err(|x| x.into());
                    assert!(reader.is_eof());
                    res
                }
            }
        }

        macro_rules! without_data {
            ($x: expr, $buf: expr) => {
                {
                    Ok($x)
                }
            }
        }

        macro_rules! decoded {
            ($x:ty, $buf:expr, $var:expr) => {
                {
                    decode!($x, $buf).map($var)
                }
            };
            ($x:ty, $buf:expr, $var:expr, $dir:expr) => {
                {
                    decode!($x, $buf).map(|x| $var($dir, x))
                }
            };
        }

        match discriminator {
            // these hold no data
            0x01 => without_data!(RawMessage::HeartbeatRequest, buf),
            0x02 => without_data!(RawMessage::HeartbeatResponse, buf),
            0x03 => without_data!(RawMessage::Ping, buf),
            0x04 => without_data!(RawMessage::Pong, buf),

            0x82 => decoded!(WriteEvents, buf, RawMessage::WriteEvents),
            0x83 => decoded!(WriteEventsCompleted, buf, RawMessage::WriteEventsCompleted),

            0xB0 => decoded!(ReadEvent, buf, RawMessage::ReadEvent),
            0xB1 => decoded!(ReadEventCompleted, buf, RawMessage::ReadEventCompleted),

            0xB2 => decoded!(ReadStreamEvents, buf, RawMessage::ReadStreamEvents, Forward),
            0xB3 => decoded!(ReadStreamEventsCompleted, buf, RawMessage::ReadStreamEventsCompleted, Forward),
            0xB4 => decoded!(ReadStreamEvents, buf, RawMessage::ReadStreamEvents, Backward),
            0xB5 => decoded!(ReadStreamEventsCompleted, buf, RawMessage::ReadStreamEventsCompleted, Backward),

            0xB6 => decoded!(ReadAllEvents, buf, RawMessage::ReadAllEvents, Forward),
            0xB7 => decoded!(ReadAllEventsCompleted, buf, RawMessage::ReadAllEventsCompleted, Forward),
            0xB8 => decoded!(ReadAllEvents, buf, RawMessage::ReadAllEvents, Backward),
            0xB9 => decoded!(ReadAllEventsCompleted, buf, RawMessage::ReadAllEventsCompleted, Backward),

            0xF0 => Ok(RawMessage::BadRequest(Cow::Borrowed(buf).into())),
            0xF1 => decoded!(NotHandled, buf, RawMessage::NotHandled),
            0xF2 => without_data!(RawMessage::Authenticate, buf),
            0xF3 => without_data!(RawMessage::Authenticated, buf),
            0xF4 => Ok(RawMessage::NotAuthenticated(Cow::Borrowed(buf).into())),
            x => Ok((x, Cow::Borrowed(buf)).into()),
        }
    }

    /// Encodes the message into the given writer.
    pub fn encode<W: io::Write>(&self, w: &mut W) -> io::Result<()> {
        use self::RawMessage::*;
        use quick_protobuf::MessageWrite;

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

        match *self {
            HeartbeatRequest |
            HeartbeatResponse |
            Ping |
            Pong |
            Authenticate |
            Authenticated => Ok(()),

            WriteEvents(ref x) => encode!(x, w),
            WriteEventsCompleted(ref x) => encode!(x, w),

            ReadEvent(ref x) => encode!(x, w),
            ReadEventCompleted(ref x) => encode!(x, w),

            ReadStreamEvents(_, ref x) => encode!(x, w),
            ReadStreamEventsCompleted(_, ref x) => encode!(x, w),

            ReadAllEvents(_, ref x) => encode!(x, w),
            ReadAllEventsCompleted(_, ref x) => encode!(x, w),

            BadRequest(ref x) => w.write_all(x.as_ref()),
            NotHandled(ref x) => encode!(x, w),
            NotAuthenticated(ref x) => w.write_all(x.as_ref()),
            Unsupported(_, ref x) => w.write_all(x),
        }
    }

    /// Returns the protocol discriminator value for the variant
    pub fn discriminator(&self) -> u8 {
        // FIXME: copied from ::Message
        use self::RawMessage::*;
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
            NotHandled(_) => 0xf1,
            Authenticate => 0xf2,
            Authenticated => 0xf3,
            NotAuthenticated(_) => 0xf4,
            Unsupported(d, _) => d,
        }
    }
}
