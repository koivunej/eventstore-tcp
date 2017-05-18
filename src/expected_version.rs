/// `ExpectedVersion` represents the different modes of optimistic locking when writing to a stream
/// using `WriteEventsBuilder`.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ExpectedVersion {
    /// No optimistic locking
    Any,
    /// Expect a stream not to exist
    NoStream,
    /// Expect exact number of events in the stream
    Exact(u32)
}

impl From<ExpectedVersion> for i32 {
    /// Returns the wire representation.
    fn from(version: ExpectedVersion) -> Self {
        use self::ExpectedVersion::*;
        match version {
            Any => -2,
            NoStream => -1,
            Exact(ver) => ver as i32
        }
    }
}
