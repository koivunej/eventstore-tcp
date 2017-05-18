/// Content type of the event `data` or `metadata`.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ContentType {
    /// Raw bytes
    Bytes,
    /// JSON values usable with projections in EventStore
    Json
}

impl Into<i32> for ContentType {
    fn into(self) -> i32 {
        match self {
            ContentType::Bytes => 0,
            ContentType::Json => 1,
        }
    }
}
