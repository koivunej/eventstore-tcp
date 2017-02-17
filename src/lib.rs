//#![feature(plugin)]
// #![plugin(protobuf_macros)]

#[macro_use]
extern crate bitflags;
// extern crate protobuf;
extern crate quick_protobuf;
extern crate uuid;
extern crate byteorder;
#[macro_use]
extern crate error_chain;
extern crate futures;
extern crate tokio_core;

#[cfg(test)]
extern crate rustc_serialize;

//use std::borrow::Cow;
use std::fmt;
use std::io;
use std::io::{Read, Write};
use std::ops::{Deref, Range};
use std::borrow::Cow;
use uuid::Uuid;
use byteorder::{ReadBytesExt, WriteBytesExt, LittleEndian};
use tokio_core::io::{Codec, EasyBuf};

// mod pb_client_messages;
mod messages;
use messages::mod_EventStore::mod_Client::mod_Messages as client_messages;
use messages::mod_EventStore::mod_Client::mod_Messages::mod_NotHandled::{NotHandledReason, MasterInfo};

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

bitflags!{
    flags TcpFlags: u8 {
        const FLAG_NONE = 0x00,
        const FLAG_AUTHENTICATED = 0x01,
        //const FLAG_TRUSTED_WRITE = 0x02, // only in core
    }
}

/// Wrapper for packet in the protocol. On the wire, packets are embedded in frames with length
/// prefix and suffix.
#[derive(Debug, PartialEq)]
pub struct Package {
    /// Possible authentication data included in the packet. `Some`` and `None` values of this will
    /// be used to generate corresponding `TcpFlags` first bit.
    pub authentication: Option<UsernamePassword>,
    /// Before sending an request to the server client generates a new random UUID using
    /// `uuid::Uuid::new_v4()` and later server will respond using the same `correlation_id`.
    pub correlation_id: Uuid,
    /// Enumeration of possible messages.
    pub message: Message,
}

/// TODO: investigate if we could at least encode Cow<'a, str>
#[derive(PartialEq, Eq)]
pub struct UsernamePassword(String, String);

impl fmt::Debug for UsernamePassword {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "({:?}, PASSWORD)", self.0)
    }
}

impl UsernamePassword {
    fn decode<R: ReadBytesExt>(buf: &mut R) -> io::Result<Self> {
        let len = buf.read_u8()?;
        let mut username = vec![0u8, len];
        buf.read_exact(&mut username[..])?;
        let username = String::from_utf8(username).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.utf8_error()))?;

        let len = buf.read_u8()?;
        let mut password = vec![0u8, len];
        buf.read_exact(&mut password[..])?;
        let password = String::from_utf8(password).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.utf8_error()))?;

        Ok(UsernamePassword(username, password))
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

    WriteEvents,
    WriteEventsCompleted(WriteEventsCompletedData),

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
pub enum WriteEventsCompletedData {
    Success {
        event_numbers: Range<i32>,
        prepare_position: Option<i64>,
        commit_position: Option<i64>,
    },
    Failure(Explanation),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Explanation {
    pub reason: OperationResult,
    pub message: Option<String>
}

impl Explanation {
    fn new(or: client_messages::OperationResult, msg: Option<String>) -> Explanation {
        Explanation { reason: or.into(), message: msg }
    }
}

/// Like `OperationResult` on the wire but does not have a success value. Explains the reason for
/// failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OperationResult {
    PrepareTimeout,
    CommitTimeout,
    ForwardTimeout,
    WrongExpectedVersion,
    StreamDeleted,
    InvalidTransaction,
    AccessDenied,
}

impl Copy for OperationResult {}

impl From<client_messages::OperationResult> for OperationResult {
    fn from(or: client_messages::OperationResult) -> OperationResult {
        use client_messages::OperationResult::*;

        match or {
            Success => panic!("Success type is invalid for this type"),
            PrepareTimeout => OperationResult::PrepareTimeout,
            CommitTimeout => OperationResult::CommitTimeout,
            ForwardTimeout => OperationResult::ForwardTimeout,
            WrongExpectedVersion => OperationResult::WrongExpectedVersion,
            StreamDeleted => OperationResult::StreamDeleted,
            InvalidTransaction => OperationResult::InvalidTransaction,
            AccessDenied => OperationResult::AccessDenied,
        }
    }
}

impl Into<client_messages::OperationResult> for OperationResult {
    fn into(self) -> client_messages::OperationResult {
        use OperationResult::*;
        match self {
            PrepareTimeout => client_messages::OperationResult::PrepareTimeout,
            CommitTimeout => client_messages::OperationResult::CommitTimeout,
            ForwardTimeout => client_messages::OperationResult::ForwardTimeout,
            WrongExpectedVersion => client_messages::OperationResult::WrongExpectedVersion,
            StreamDeleted => client_messages::OperationResult::StreamDeleted,
            InvalidTransaction => client_messages::OperationResult::InvalidTransaction,
            AccessDenied => client_messages::OperationResult::AccessDenied
        }
    }
}

impl<'a> From<client_messages::WriteEventsCompleted<'a>> for WriteEventsCompletedData {
    fn from(wec: client_messages::WriteEventsCompleted<'a>) -> Self {
        use client_messages::OperationResult::*;
        match wec.result.expect("Required field was not present in WroteEventsComplete") {
            Success => {
                WriteEventsCompletedData::Success {
                    // off-by one: Range is [start, end)
                    event_numbers: wec.first_event_number..wec.last_event_number + 1,
                    prepare_position: wec.prepare_position,
                    commit_position: wec.commit_position,
                }
            }
            x => WriteEventsCompletedData::Failure(Explanation::new(x, wec.message.map(|x| x.into_owned()))),
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

impl<'a> AsMessageWrite<client_messages::WriteEventsCompleted<'a>> for WriteEventsCompletedData {
    fn as_message_write(&self) -> client_messages::WriteEventsCompleted<'a> {
        use WriteEventsCompletedData::*;
        match *self {
            Success { ref event_numbers, prepare_position, commit_position } => {
                client_messages::WriteEventsCompleted {
                    result: Some(client_messages::OperationResult::Success),
                    message: None,
                    first_event_number: event_numbers.start,
                    last_event_number: event_numbers.end - 1,
                    prepare_position: prepare_position,
                    commit_position: commit_position
                }
            },
            Failure(Explanation { reason, ref message }) => {
                client_messages::WriteEventsCompleted {
                    result: Some(reason.into()),
                    message: message.clone().map(Cow::Owned),
                    first_event_number: -1,
                    last_event_number: -1,
                    prepare_position: None,
                    commit_position: None,
                }
            }
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

            //0x82 => Message::WriteEvents(parse!(client_messages::WriteEvents, buf.as_slice())?.into()),
            0x83 => Message::WriteEventsCompleted(parse!(client_messages::WriteEventsCompleted, buf.as_slice())?.into()),

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
                    .map(|bytes| parse!(MasterInfo, bytes.deref()).map(|x| Self::owned_master_info(x)).map(Option::Some).unwrap_or(None))
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

    fn owned_master_info<'a>(m: MasterInfo<'a>) -> MasterInfo<'static> {
        MasterInfo {
            external_tcp_address: Cow::Owned(m.external_tcp_address.into_owned()),
            external_tcp_port: m.external_tcp_port,
            external_http_address: Cow::Owned(m.external_http_address.into_owned()),
            external_http_port: m.external_http_port,
            external_secure_tcp_address: m.external_secure_tcp_address.map(|x| Cow::Owned(x.into_owned())),
            external_secure_tcp_port: m.external_secure_tcp_port
        }
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

        Ok(match *self {
            HeartbeatRequest |
            HeartbeatResponse |
            Ping |
            Pong |
            Authenticate |
            Authenticated |
            NotAuthenticated => (),

            WriteEvents => (), //x.encode(w)?,

            WriteEventsCompleted(ref x) => x.encode(w)?,
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

            WriteEvents => 0x82,
            WriteEventsCompleted(_) => 0x83,

            BadRequest(_) => 0xf0,
            NotHandled(..) => 0xf1,
            Authenticate => 0xf2,
            Authenticated => 0xf3,
            NotAuthenticated => 0xf4
        }
    }
}

pub struct PackageCodec;

impl PackageCodec {
    fn decode_inner(&mut self, buf: &mut EasyBuf) -> io::Result<Option<Package>> {
        if buf.len() < 4 + 1 + 1 + 16 {
            return Ok(None);
        }

        let len = {
            let mut cursor = io::Cursor::new(buf.as_slice());
            cursor.read_u32::<LittleEndian>()?
        } as usize;

        if len < 18 {
            panic!("length is too little: {}", len);
        }

        if buf.len() < len + 4 {
            return Ok(None);
        }

        let mut frame = buf.clone();
        frame.drain_to(4);
        frame.split_off(len);

        let decoded_frame = self.decode_body(&mut frame);

        match decoded_frame {
            Ok((c, a, m)) => {
                buf.drain_to(4 + len);
                Ok(Some(Package {
                    correlation_id: c,
                    authentication: a,
                    message: m,
                }))
            }
            Err(e) => Err(e),
        }
    }

    fn decode_body(&mut self,
                   buf: &mut EasyBuf)
                   -> io::Result<(Uuid, Option<UsernamePassword>, Message)> {
        let (d, c, a) = self.decode_header(buf)?;
        let message = Message::decode(d, buf)?;
        Ok((c, a, message))
    }

    fn decode_header(&mut self,
                     buf: &mut EasyBuf)
                     -> io::Result<(u8, Uuid, Option<UsernamePassword>)> {
        let (d, c, a, pos) = {
            let mut cursor = io::Cursor::new(buf.as_slice());
            let discriminator = cursor.read_u8()?;
            let flags = cursor.read_u8()?;
            let flags = match TcpFlags::from_bits(flags) {
                Some(flags) => flags,
                None => bail!(ErrorKind::InvalidFlags(flags)),
            };

            let correlation_id = {
                let mut uuid_bytes = [0u8; 16];
                cursor.read_exact(&mut uuid_bytes)?;
                // this should only err if len is not 16
                Uuid::from_bytes(&uuid_bytes).unwrap()
            };

            let authentication = if flags.contains(FLAG_AUTHENTICATED) {
                Some(UsernamePassword::decode(&mut cursor)?)
            } else {
                None
            };

            (discriminator, correlation_id, authentication, cursor.position() as usize)
        };

        buf.drain_to(pos);
        Ok((d, c, a))
    }
}

impl Codec for PackageCodec {
    type In = Package;
    type Out = Package;

    fn decode(&mut self, buf: &mut EasyBuf) -> io::Result<Option<Self::In>> {
        self.decode_inner(buf).map_err(|e| e.into())
    }

    fn encode(&mut self, msg: Package, buf: &mut Vec<u8>) -> io::Result<()> {
        // not sure how to make this without tmp vec
        let mut cursor = io::Cursor::new(Vec::new());

        let mut flags = FLAG_NONE;
        if msg.authentication.is_some() {
            flags.insert(FLAG_AUTHENTICATED);
        }

        cursor.write_u32::<LittleEndian>(0)?; // placeholder for prefix
        cursor.write_u8(msg.message.discriminator())?;
        cursor.write_u8(flags.bits())?;
        cursor.write_all(msg.correlation_id.as_bytes())?;
        if flags.contains(FLAG_AUTHENTICATED) {
            msg.authentication
                .expect("According to flag authentication token is present")
                .encode(&mut cursor)?;
        }

        msg.message.encode(&mut cursor)?;

        let at_end = cursor.position();
        let len = at_end as u32 - 4;

        cursor.set_position(0);
        cursor.write_u32::<LittleEndian>(len)?;

        let tmp = cursor.into_inner();
        buf.extend(tmp);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;
    use rustc_serialize::hex::{ToHex, FromHex};
    use tokio_core::io::Codec;
    use uuid::Uuid;
    use super::{PackageCodec, Package, FLAG_NONE, Message, WriteEventsCompletedData};

    #[test]
    fn decode_ping() {
        test_decoding_hex("1200000003007b50a1b034b9224e8f9d708c394fab2d",
                          PackageCodec,
                          Package {
                              authentication: None,
                              correlation_id:
                                  Uuid::parse_str("7b50a1b0-34b9-224e-8f9d-708c394fab2d").unwrap(),
                              message: Message::Ping,
                          });
    }

    #[test]
    fn decode_ping_with_junk() {
        test_decoding_hex("1300000003007b50a1b034b9224e8f9d708c394fab2d00",
                          PackageCodec,
                          Package {
                              authentication: None,
                              correlation_id:
                                  Uuid::parse_str("7b50a1b0-34b9-224e-8f9d-708c394fab2d").unwrap(),
                              message: Message::Ping,
                          });
    }

    #[test]
    fn encode_ping() {
        test_encoding_hex("1200000003007b50a1b034b9224e8f9d708c394fab2d",
                          PackageCodec,
                          Package {
                              authentication: None,
                              correlation_id:
                                  Uuid::parse_str("7b50a1b0-34b9-224e-8f9d-708c394fab2d").unwrap(),
                              message: Message::Ping,
                          });
    }

    #[test]
    fn decode_unknown_discriminator() {
        use std::io;

        let err = PackageCodec.decode(&mut ("12000000ff007b50a1b034b9224e8f9d708c394fab2d"
                .to_string()
                .from_hex()
                .unwrap()
                .into()))
            .unwrap_err();

        assert_eq!(err.kind(), io::ErrorKind::Other);
        let err = err.into_inner();
        match err {
            Some(inner) => {
                match *inner.downcast::<super::errors::Error>().unwrap() {
                    super::errors::Error(super::errors::ErrorKind::UnsupportedDiscriminator(0xff), _) => { /* good */ }
                    x => panic!("unexpected errorkind: {:?}", x),
                }
            }
            x => panic!("unexpected inner error: {:?}", x),
        }
    }

    #[test]
    fn decode_write_events_completed() {
        let input = "2200000083009b59d8734e9fd84eb8a421f2666a3aa40800181e20272884d6bc563084d6bc56";
        test_decoding_hex(input,
                          PackageCodec,
                          Package {
                              authentication: None,
                              correlation_id:
                                  Uuid::parse_str("9b59d873-4e9f-d84e-b8a4-21f2666a3aa4").unwrap(),
                              message: Message::WriteEventsCompleted(WriteEventsCompletedData::Success {
                                  event_numbers: 30..40,
                                  prepare_position: Some(181349124),
                                  commit_position: Some(181349124)
                              })
                          });
    }

    /*#[test]
    fn decode_wec2() {
        use protobuf;
        use std::io;;
        use pb_client_messages::WriteEventsCompleted as PBWEC;
        let input = "0800181e20272884d6bc563084d6bc56".to_string().from_hex().unwrap();
        println!("{:?}", input);
        let mut cursor = io::Cursor::new(input);
        let parsed = protobuf::parse_from_reader::<PBWEC>(&mut cursor).unwrap();
        println!("{:?}", parsed);
    }*/

    #[test]
    fn encode_write_events_completed() {
        test_encoding_hex("2200000083009b59d8734e9fd84eb8a421f2666a3aa40800181e20272884d6bc563084d6bc56",
                          PackageCodec,
                          Package {
                              authentication: None,
                              correlation_id:
                                  Uuid::parse_str("9b59d873-4e9f-d84e-b8a4-21f2666a3aa4").unwrap(),
                              message: Message::WriteEventsCompleted(WriteEventsCompletedData::Success {
                                  event_numbers: 30..40,
                                  prepare_position: Some(181349124),
                                  commit_position: Some(181349124)
                              })
                          });

    }

    fn test_decoding_hex<C: Codec>(input: &str, codec: C, expected: C::In)
        where C::In: Debug + PartialEq
    {
        test_decoding(input.to_string().from_hex().unwrap(), codec, expected);
    }

    fn test_decoding<C: Codec>(input: Vec<u8>, mut codec: C, expected: C::In)
        where C::In: Debug + PartialEq
    {
        // decode whole buffer
        {
            let mut buf = input.clone().into();
            let item = codec.decode(&mut buf).unwrap().unwrap();

            assert_eq!(item, expected);
            assert_eq!(buf.len(), 0, "decoding correctly sized buffer left bytes");
        }

        // decoding partial buffer consumes no bytes
        for len in 1..(input.len() - 1) {
            let mut part = input.clone();
            part.truncate(len);

            let mut buf = part.into();
            assert!(codec.decode(&mut buf).unwrap().is_none());
            assert_eq!(buf.len(), len, "decoding partial buffer consumed bytes");
        }

        // decoding a too long buffer consumes no extra bytes
        {
            let mut input = input.clone();
            let len = input.len();
            input.extend(vec![0u8; len]);
            let mut buf = input.into();
            let item = codec.decode(&mut buf).unwrap().unwrap();
            assert_eq!(item, expected);
            assert_eq!(buf.len(), len, "decoding oversized buffer overused bytes");
        }
    }

    fn test_encoding_hex<C: Codec>(input: &str, codec: C, expected: C::Out)
        where C::In: Debug + PartialEq
    {
        test_encoding(input.to_string().from_hex().unwrap(), codec, expected);
    }

    fn test_encoding<C: Codec>(input: Vec<u8>, mut codec: C, expected: C::Out)
        where C::In: Debug + PartialEq
    {
        let mut buf = Vec::new();
        codec.encode(expected, &mut buf).unwrap();
        assert_eq!(buf.as_slice(),
                   input.as_slice(),
                   "encoding did not yield same");
    }
}
