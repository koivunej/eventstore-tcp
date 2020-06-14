//! Raw module contains the enumeration `RawMessage` and raw decoding and encoding functionality.
//! There should not be need to handle `RawMessage` values directly but if there is ever a bug,
//! using the raw messages should still work.

use std::io;
use std::str;
use std::borrow::Cow;
use std::convert::TryFrom;
use quick_protobuf;

pub mod client_messages;
pub use self::client_messages::{EventRecord, WriteEvents, WriteEventsCompleted, ReadEvent, ReadEventCompleted, ReadStreamEvents, ReadStreamEventsCompleted, ReadAllEvents, ReadAllEventsCompleted, NotHandled, DeleteStream, DeleteStreamCompleted, OperationResult};

use adapted;

use errors::Error;
use ReadDirection;

/// Enumeration much like the `adapted::AdaptedMessage` for all the messages in the protocol.
/// `RawMessage` variants are literally the raw protobuf messages and are not validated in any way on
/// top the protobuf decoding.
#[derive(Debug, PartialEq, Clone, From, IntoOwned)]
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

    /// Request to delete a stream
    DeleteStream(DeleteStream<'a>),
    /// Response to previous stream deletion request
    DeleteStreamCompleted(DeleteStreamCompleted<'a>),

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
#[doc(hidden)]
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
#[derive(Debug, PartialEq, Clone, From, Into, IntoOwned)]
pub struct NotAuthenticatedPayload<'a>(Cow<'a, [u8]>);

impl<'a> AsRef<[u8]> for NotAuthenticatedPayload<'a> {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl<'a> ByteWrapper<'a> for NotAuthenticatedPayload<'a> {
    type ConversionErr = Error;
}

/// Newtype for an arbitary BadRequest "info", which could be
/// UTF8 string.
#[derive(Debug, PartialEq, Clone, From, Into, IntoOwned)]
pub struct BadRequestPayload<'a>(Cow<'a, [u8]>);

impl<'a> AsRef<[u8]> for BadRequestPayload<'a> {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl<'a> ByteWrapper<'a> for BadRequestPayload<'a> {
    type ConversionErr = Error;
}

impl<'a> RawMessage<'a> {

    /// Attempt to convert a raw message into an adapted one
    pub fn try_adapt(self) -> Result<adapted::AdaptedMessage<'a>, (Self, Error)> {
        adapted::AdaptedMessage::try_from(self)
    }

    /// Decodes the message from the buffer without any cloning.
    pub fn decode(discriminator: u8, buf: &'a [u8]) -> io::Result<RawMessage<'a>> {
        use ReadDirection::{Forward, Backward};

        macro_rules! decode {
            ($x:ty, $buf:expr) => {
                {
                    let mut reader = ::quick_protobuf::reader::BytesReader::from_bytes($buf);
                    let res: Result<$x, io::Error> = <$x>::from_reader(&mut reader, $buf)
                        .map_err(|x| x.into());
                    assert!(reader.is_eof(), "reader did not decode everything: {:?}, bytes:\n{:?}", reader, Hexdump { bytes: $buf });
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

            0x8A => decoded!(DeleteStream, buf, RawMessage::DeleteStream),
            0x8B => decoded!(DeleteStreamCompleted, buf, RawMessage::DeleteStreamCompleted),

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

            DeleteStream(ref x) => encode!(x, w),
            DeleteStreamCompleted(ref x) => encode!(x, w),

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

            DeleteStream(_) => 0x8A,
            DeleteStreamCompleted(_) => 0x8B,

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

use std::fmt;

struct Hexdump<'x> {
    bytes: &'x [u8],
}

impl<'x> Hexdump<'x> {
    fn format_ascii(&self, fmt: &mut fmt::Formatter, start: usize, end: usize) -> fmt::Result {
        let group_len = 8;
        let mut char_count = 0;

        for i in start..end {
            let b = self.bytes[i];

            if 0x20 <= b && b < 0x7e {
                write!(fmt, "{}", b as char)?;
            } else {
                write!(fmt, ".")?;
            }

            char_count += 1;

            if i < end - 1 && char_count % group_len == 0 {
                write!(fmt, " ")?;
            }
        }

        Ok(())
    }
}

impl<'x> fmt::Debug for Hexdump<'x> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {

        let group_len = 4;
        let groups_per_line = 4;

        let bytes_per_line = group_len * groups_per_line;

        let mut count = 0;

        let prefix_len = (self.bytes.len() as f64).log10().ceil() as usize;


        for b in self.bytes[..].iter() {
            if count % bytes_per_line == 0 {
                write!(fmt, "0x{:01$x}: ", count, prefix_len)?;
            }

            let last = count == self.bytes.len() - 1;
            write!(fmt, "{:02x}", b)?;
            count += 1;
            if count % bytes_per_line == 0 {

                write!(fmt, " | ")?;

                self.format_ascii(fmt, count - 4*4, count)?;

                writeln!(fmt)?;
            } else if !last {
                if count % group_len == 0 {
                    write!(fmt, "  ")?;
                } else {
                    write!(fmt, " ")?;
                }
            }
        }

        if count % bytes_per_line > 0 {

            let offset = count % 4;
            let end = (bytes_per_line - (count % bytes_per_line)) + offset;

            for i in offset..end {
                write!(fmt, "  ")?;

                if i % group_len == 0 {
                    write!(fmt, "  ")?;
                } else {
                    write!(fmt, " ")?;
                }
            }

            write!(fmt, " | ")?;

            self.format_ascii(fmt, count - (count % bytes_per_line), count)?;

            writeln!(fmt)?;
        }

        Ok(())
    }
}

#[test]
fn test_hexdump_even() {
    let bytes = (0u8..10).into_iter().cycle().take(20).collect::<Vec<u8>>();

    let out = format!("{:?}", Hexdump { bytes: &bytes[..] });
    println!("\n{}", out);

    assert_eq!(out, "0x00: 00 01 02 03  04 05 06 07  08 09 00 01  02 03 04 05 | ........ ........\n0x10: 06 07 08 09                                        | ....\n");
}

#[test]
fn test_hexdump_odd() {
    let bytes = (0u8..10).into_iter().cycle().take(21).collect::<Vec<u8>>();

    let out = format!("{:?}", Hexdump { bytes: &bytes[..] });
    println!("\n{}", out);

    assert_eq!(out, "0x00: 00 01 02 03  04 05 06 07  08 09 00 01  02 03 04 05 | ........ ........\n0x10: 06 07 08 09  00                                    | .....\n");
}
