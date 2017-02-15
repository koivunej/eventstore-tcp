//#![feature(plugin)]
// #![plugin(protobuf_macros)]

#[macro_use]
extern crate bitflags;
extern crate protobuf;
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
use uuid::Uuid;
use byteorder::{ReadBytesExt, WriteBytesExt, LittleEndian};
use tokio_core::io::{Codec, EasyBuf};

// mod ClientMessageDtos;

pub mod errors {
    use std::str;
    use std::string;
    use std::io;

    error_chain! {
        foreign_links {
            Utf8(str::Utf8Error);
            Io(io::Error);
        }

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
                Error(ErrorKind::Io(e), _) => e,
                e => io::Error::new(io::ErrorKind::Other, e),
            }
        }
    }

    impl From<string::FromUtf8Error> for Error {
        fn from(e: string::FromUtf8Error) -> Self {
            e.utf8_error().into()
        }
    }

    impl Into<io::Error> for ErrorKind {
        fn into(self) -> io::Error {
            Error::from(self).into()
        }
    }
}

use self::errors::ErrorKind;

/*
enum ErrorKind {
    InvalidFlags(u8),
    UnsupportedDiscriminator(u8),
}
*/

bitflags!{
    flags TcpFlags: u8 {
        const FLAG_NONE = 0x00,
        const FLAG_AUTHENTICATED = 0x01,
        //const FLAG_TRUSTED_WRITE = 0x02, // only in core
    }
}

/// Wrapper for packet in the protocol. On the wire, packets are embedded in frames with length
/// prefix and suffix.
#[derive(Debug, PartialEq, Eq)]
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

#[derive(Debug, PartialEq, Eq)]
pub enum Message {
    HeartbeatRequest,
    HeartbeatResponse,
    Ping,
    Pong,

    WriteEvents,
    WriteEventsCompleted,
}

impl Message {
    fn decode(discriminator: u8, buf: &mut EasyBuf) -> io::Result<Message> {
        Ok(match discriminator {
            // these hold no data
            0x01 => Self::without_data(Message::HeartbeatRequest, buf),
            0x02 => Self::without_data(Message::HeartbeatResponse, buf),
            0x03 => Self::without_data(Message::Ping, buf),
            0x04 => Self::without_data(Message::Pong, buf),

            0x82 => Self::without_data(Message::WriteEvents, buf),
            0x83 => Self::without_data(Message::WriteEventsCompleted, buf),

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

            0xF0 => { /* badrequest */ }
            0xF1 => { /* not handled */ }
            0xF2 => { /* authenticate */ }
            0xF3 => { /* authenticated */ }
            0xF4 => { /* not authenticated */ }*/
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

    fn encode<W: WriteBytesExt>(&self, _: &mut W) -> io::Result<()> {
        use Message::*;
        match *self {
            HeartbeatRequest | HeartbeatResponse | Ping | Pong => Ok(()),
            WriteEvents | WriteEventsCompleted => Ok(()),
        }
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
            WriteEventsCompleted => 0x83,
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
    use tokio_core::io::{Codec, EasyBuf};
    use uuid::Uuid;
    use super::{PackageCodec, Package, FLAG_NONE, Message};

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
                              message: Message::WriteEventsCompleted,
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
