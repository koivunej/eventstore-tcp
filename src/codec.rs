//! `codec` module contains the `Package` (frame) decoding and an `tokio_core::io::Codec`
//! implementation.

use std::io::{self, Read, Write};
use uuid::Uuid;
use byteorder::{ReadBytesExt, WriteBytesExt, LittleEndian};
use tokio_io::codec::{Encoder, Decoder};
use bytes::{BytesMut, BufMut};

use errors::ErrorKind;
use package::Package;
use {UsernamePassword};
use raw::RawMessage;

bitflags!{
    /// `TcpFlags` describes if optional fields (authentication) is present.
    pub flags TcpFlags: u8 {
        /// No authentication information present
        const FLAG_NONE = 0x00,
        /// Package contains authentication
        const FLAG_AUTHENTICATED = 0x01,
        //const FLAG_TRUSTED_WRITE = 0x02, // only in core
    }
}

/// Stateless simple PackageCodec
pub struct PackageCodec;

impl PackageCodec {
    fn decode_inner(&mut self, buf: &mut BytesMut) -> io::Result<Option<Package>> {
        if buf.len() < 4 + 1 + 1 + 16 {
            return Ok(None);
        }

        let len = io::Cursor::new(&buf[0..4]).read_u32::<LittleEndian>()? as usize;

        if len < 18 {
            panic!("length is too little: {}", len);
        }

        if buf.len() < len + 4 {
            return Ok(None);
        }

        let decoded_frame = self.decode_body(&buf[4..(4 + len)]);
        decoded_frame.and_then(|(c, a, m)| {
            buf.split_to(4 + len);
            Ok(Some(Package {
                correlation_id: c,
                authentication: a,
                message: m.into(),
            }))
        })
    }

    fn decode_body(&mut self, buf: &[u8]) -> io::Result<(Uuid, Option<UsernamePassword>, RawMessage<'static>)> {
        let (d, c, a, pos) = self.decode_header(buf)?;
        let message = RawMessage::decode(d, &buf[pos..])?.into_owned();
        Ok((c, a, message))
    }

    fn decode_header(&mut self, buf: &[u8]) -> io::Result<(u8, Uuid, Option<UsernamePassword>, usize)> {
        let (d, c, a, pos) = {
            let mut cursor = io::Cursor::new(buf);
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

        Ok((d, c, a, pos))
    }

    #[doc(hidden)]
    pub fn encode_parts<'a>(&self, cursor: &mut io::Cursor<Vec<u8>>, correlation_id: &Uuid, authentication: Option<&UsernamePassword>, raw: &RawMessage<'a>) -> io::Result<()> {
        let mut flags = FLAG_NONE;
        if authentication.is_some() {
            flags.insert(FLAG_AUTHENTICATED);
        }

        cursor.write_u32::<LittleEndian>(0)?; // placeholder for prefix
        cursor.write_u8(raw.discriminator())?;
        cursor.write_u8(flags.bits())?;
        cursor.write_all(correlation_id.as_bytes())?;
        if flags.contains(FLAG_AUTHENTICATED) {
            authentication
                .expect("According to flag authentication token is present")
                .encode(cursor)?;
        } else {
            assert!(authentication.is_none());
        }

        raw.encode(cursor)?;

        let at_end = cursor.position();
        let len = at_end as u32 - 4;

        cursor.set_position(0);
        cursor.write_u32::<LittleEndian>(len)?;
        Ok(())
    }
}

impl Decoder for PackageCodec {
    type Item = Package;
    type Error = io::Error;

    fn decode(&mut self, buf: &mut BytesMut) -> io::Result<Option<Self::Item>> {
        self.decode_inner(buf).map_err(|e| e.into())
    }
}

impl Encoder for PackageCodec {
    type Item = Package;
    type Error = io::Error;

    fn encode(&mut self, msg: Package, buf: &mut BytesMut) -> io::Result<()> {
        let mut cursor = io::Cursor::new(Vec::new());

        self.encode_parts(&mut cursor, &msg.correlation_id, msg.authentication.as_ref(), &msg.message)?;

        let tmp = cursor.into_inner();
        buf.put_slice(&tmp);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;
    use rustc_serialize::hex::FromHex;
    use tokio_io::codec::{Decoder, Encoder};
    use uuid::Uuid;
    use super::{PackageCodec};
    use package::Package;
    use raw::RawMessage;
    use raw::client_messages::{WriteEventsCompleted, OperationResult};

    #[test]
    fn decode_ping() {
        test_decoding_hex("1200000003007b50a1b034b9224e8f9d708c394fab2d",
                          PackageCodec,
                          Package {
                              authentication: None,
                              correlation_id:
                                  Uuid::parse_str("7b50a1b0-34b9-224e-8f9d-708c394fab2d").unwrap(),
                              message: RawMessage::Ping.into(),
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
                              message: RawMessage::Ping.into(),
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
                              message: RawMessage::Ping.into(),
                          });
    }

    #[test]
    fn decode_unknown_discriminator() {
        use std::borrow::Cow;

        let err = PackageCodec.decode(&mut ("12000000ff007b50a1b034b9224e8f9d708c394fab2d"
                .to_string()
                .from_hex()
                .unwrap()
                .into()))
            .unwrap() // Result
            .unwrap();// Option

        assert_eq!(err, Package {
            authentication: None,
            correlation_id: Uuid::parse_str("7b50a1b0-34b9-224e-8f9d-708c394fab2d").unwrap(),
            message: RawMessage::Unsupported(255, Cow::Owned(vec![])).into()
        });
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
                              message: RawMessage::WriteEventsCompleted(WriteEventsCompleted {
                                  result: Some(OperationResult::Success),
                                  message: None,
                                  first_event_number: 30,
                                  last_event_number: 39,
                                  prepare_position: Some(181349124),
                                  commit_position: Some(181349124)
                              }).into()
                          });
    }

    #[test]
    fn encode_write_events_completed() {
        test_encoding_hex("2200000083009b59d8734e9fd84eb8a421f2666a3aa40800181e20272884d6bc563084d6bc56",
                          PackageCodec,
                          Package {
                              authentication: None,
                              correlation_id:
                                  Uuid::parse_str("9b59d873-4e9f-d84e-b8a4-21f2666a3aa4").unwrap(),
                              message: RawMessage::WriteEventsCompleted(WriteEventsCompleted {
                                  result: Some(OperationResult::Success),
                                  message: None,
                                  first_event_number: 30,
                                  last_event_number: 39,
                                  prepare_position: Some(181349124),
                                  commit_position: Some(181349124)
                              }).into()
                          });

    }

    #[test]
    fn decode_authenticated_package() {
        use bytes::BytesMut;
        use auth::UsernamePassword;

        let mut buf = BytesMut::with_capacity(1024);
        let id = Uuid::new_v4();

        let msg = Package {
            correlation_id: id,
            authentication: Some(UsernamePassword::new("foobar", "abbacd")),
            message: RawMessage::Ping,
        };

        PackageCodec.encode(msg.clone(), &mut buf).unwrap();

        let decoded = PackageCodec.decode(&mut buf).unwrap().unwrap();

        assert_eq!(msg, decoded);
    }

    fn test_decoding_hex<C: Decoder>(input: &str, codec: C, expected: C::Item)
        where C::Item: Debug + PartialEq, C::Error: Debug
    {
        test_decoding(input.to_string().from_hex().unwrap(), codec, expected);
    }

    fn test_decoding<C: Decoder>(input: Vec<u8>, mut codec: C, expected: C::Item)
        where C::Item: Debug + PartialEq, C::Error: Debug
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

    fn test_encoding_hex<C: Encoder>(input: &str, codec: C, expected: C::Item)
        where C::Item: Debug + PartialEq, C::Error: Debug
    {
        test_encoding(input.to_string().from_hex().unwrap(), codec, expected);
    }

    fn test_encoding<C: Encoder>(input: Vec<u8>, mut codec: C, expected: C::Item)
        where C::Item: Debug + PartialEq, C::Error: Debug
    {
        use bytes::BytesMut;

        let mut buf = BytesMut::with_capacity(input.len());
        codec.encode(expected, &mut buf).unwrap();
        assert_eq!(&buf[..],
                   &input[..],
                   "encoding did not yield same");
    }
}
