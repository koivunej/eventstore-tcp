use std::borrow::Cow;
use raw::client_messages::mod_ReadStreamEventsCompleted::ReadStreamResult;
use raw::client_messages::ResolvedIndexedEvent;
use {StreamVersion, EventNumber};

/// Successful response to a `Message::ReadStreamEvents`.
#[derive(Debug, PartialEq, Clone)]
pub struct ReadStreamCompleted<'a> {
    /// The actual events returned by the server. Subject to `resolve_link_tos` setting on the read
    /// request.
    pub events: Vec<ResolvedIndexedEvent<'a>>,
    /// `EventNumber` for a query for the next page in the same direction, `None` if start has been
    /// reached when reading backwards. When reading forwards, this will never be `None` as new
    /// events might have appeared while receiving this response.
    pub next_page: Option<EventNumber>,
    /// Event number of the last event
    pub last_event_number: StreamVersion,
    /// Has the end of the stream been reached (or could more events be read immediatedly)
    pub end_of_stream: bool,

    /// Last commit position of the last event. Not public as there is currently no type for an
    /// positive i64 (0 < x < i64). Also, not sure how to explain the use of this property.
    pub last_commit_position: i64,
}

/// Non-success projection of the `ReadStreamResult` enum on the wire representing
/// a failed `ReadStreamCompleted` request.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ReadStreamError<'a> {
    /// Stream was not found
    NoStream,
    /// Stream has been deleted
    StreamDeleted,
    /// Unknown when this happens: it is not retuned when reading past the last event in a stream
    /// forwards.
    NotModified,
    /// Other error
    Error(Option<Cow<'a, str>>),
    /// Access was denied (no credentials provided or insufficient permissions)
    AccessDenied,
}

impl<'a> From<(ReadStreamResult, Option<Cow<'a, str>>)> for ReadStreamError<'a> {
    fn from((res, err): (ReadStreamResult, Option<Cow<'a, str>>)) -> Self {
        use self::ReadStreamResult::*;
        match res {
            Success => unreachable!(),
            NoStream => ReadStreamError::NoStream,
            StreamDeleted => ReadStreamError::StreamDeleted,
            NotModified => ReadStreamError::NotModified,
            Error => ReadStreamError::Error(err),
            AccessDenied => ReadStreamError::AccessDenied,
        }
    }
}
