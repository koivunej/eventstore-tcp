use std::borrow::Cow;
use std::fmt;
use std::io;
use byteorder::{ReadBytesExt, WriteBytesExt};

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

    pub fn decode<R: ReadBytesExt>(buf: &mut R) -> io::Result<Self> {
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

    pub fn encode<W: WriteBytesExt>(&self, buf: &mut W) -> io::Result<usize> {
        // TODO: new that disallows too long strings
        buf.write_u8(self.0.len() as u8)?;
        buf.write_all(self.0.as_bytes())?;
        buf.write_u8(self.1.len() as u8)?;
        buf.write_all(self.1.as_bytes())?;

        Ok(1 + self.0.len() + 1 + self.1.len())
    }
}

