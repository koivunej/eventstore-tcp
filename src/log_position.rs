use std::convert::TryFrom;
use {Error, ErrorKind};

/// Global unique position in the EventStore, used when reading all events.
/// Range: `-1..i64::max_value()`
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogPosition {
    /// The first event ever
    First,
    /// Exact position
    Exact(u64),
    /// The last event written to the database at the moment
    Last,
}

impl TryFrom<i64> for LogPosition {
    type Error = Error;

    fn try_from(val: i64) -> Result<LogPosition, Self::Error> {
        match val {
            0 => Ok(LogPosition::First),
            -1 => Ok(LogPosition::Last),
            pos if pos > 0 => Ok(LogPosition::Exact(pos as u64)),
            invalid => Err(ErrorKind::UnderflowLogPosition(invalid).into())
        }
    }
}

impl TryFrom<u64> for LogPosition {
    type Error = Error;

    fn try_from(val: u64) -> Result<LogPosition, Self::Error> {
        match val {
            0 => Ok(LogPosition::First),
            pos if pos <= i64::max_value() as u64 => Ok(LogPosition::Exact(pos)),
            invalid => Err(ErrorKind::OverflowLogPosition(invalid).into())
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
