use std::convert::TryFrom;
use {Error, ErrorKind};

/// `StreamVersion` represents the valid values for a stream version which is the same as the
/// event number of the latest event. As such, values are non-negative integers up to
/// `i32::max_value`. Negative values of `i32` have special meaning in the protocol, and are
/// restricted from being used with this type.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct StreamVersion(u32);

impl From<StreamVersion> for u32 {
    fn from(stream_version: StreamVersion) -> Self {
        stream_version.0
    }
}

impl From<StreamVersion> for i32 {
    fn from(stream_version: StreamVersion) -> Self {
        stream_version.0 as i32
    }
}

impl TryFrom<u32> for StreamVersion {
    type Error = Error;

    fn try_from(ver: u32) -> Result<Self, Self::Error> {
        if ver < i32::max_value() as u32 {
            Ok(StreamVersion(ver))
        } else {
            Err(ErrorKind::InvalidStreamVersion(ver as i32).into())
        }
    }
}

impl TryFrom<i32> for StreamVersion {
    type Error = Error;

    fn try_from(ver: i32) -> Result<Self, Self::Error> {
        if ver >= 0 {
            Ok(StreamVersion(ver as u32))
        } else {
            Err(ErrorKind::InvalidStreamVersion(ver).into())
        }
    }
}
