use StreamVersion;

/// `EventNumber` is similar to `StreamVersion` and `ExpectedVersion` but is used when specifying a
/// position to read from in the stream. Allows specifying the first or last (when reading
/// backwards) event in addition to exact event number.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum EventNumber {
    /// The first event in a stream
    First,
    /// Exactly the given event number
    Exact(StreamVersion),
    /// The last event in a stream
    Last,
}

impl From<StreamVersion> for EventNumber {
    fn from(ver: StreamVersion) -> Self {
        EventNumber::Exact(ver)
    }
}

impl From<EventNumber> for i32 {
    /// Returns the wire representation.
    fn from(number: EventNumber) -> Self {
        use self::EventNumber::*;
        match number {
            First => 0,
            Exact(x) => x.into(),
            Last => -1
        }
    }
}
