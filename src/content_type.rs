/// Content type of the event `data` or `metadata`.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ContentType {
    /// Raw bytes
    Bytes,
    /// JSON values usable with projections in EventStore
    Json
}

impl From<ContentType> for i32 {
    fn from(content_type: ContentType) -> Self {
        match content_type {
            ContentType::Bytes => 0,
            ContentType::Json => 1,
        }
    }
}
