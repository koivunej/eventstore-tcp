use std::borrow::Cow;
use client_messages::ReadStreamEventsCompleted;
use client_messages::mod_ReadStreamEventsCompleted::ReadStreamResult;
use client_messages::ResolvedIndexedEvent;
use {Message, StreamVersion, EventNumber, ReadDirection};

/// Successful response to a `Message::ReadStreamEvents`.
#[derive(Debug, PartialEq, Clone)]
pub struct ReadStreamSuccess {
    /// The actual events returned by the server. Subject to `resolve_link_tos` setting on the read
    /// request.
    pub events: Vec<ResolvedIndexedEvent<'static>>,
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
    last_commit_position: i64,
}

impl<'a> From<(ReadDirection, ReadStreamEventsCompleted<'a>)> for Message {
    fn from((dir, completed): (ReadDirection, ReadStreamEventsCompleted<'a>)) -> Message {
        use client_messages_ext::ResolvedIndexedEventExt;

        match completed.result {
            Some(ReadStreamResult::Success) => {
                let next_page = if dir == ReadDirection::Backward && completed.next_event_number < 0 {
                    None
                } else {
                    EventNumber::from_i32_opt(completed.next_event_number)
                };

                // TODO: from_i32 can still panic but haven't found a legitimate situation

                Message::ReadStreamEventsCompleted(dir, Ok(ReadStreamSuccess {
                    events: completed.events.into_iter().map(|x| x.into_owned()).collect(),
                    next_page: next_page,
                    last_event_number: StreamVersion::from_i32(completed.last_event_number),
                    end_of_stream: completed.is_end_of_stream,
                    last_commit_position: completed.last_commit_position,
                }))
            },

            Some(err) => {
                // TODO: last_commit_position has readable value which is discarded here
                Message::ReadStreamEventsCompleted(dir, Err((err, completed.error).into()))
            },

            None => panic!("No result found from ReadStreamEventsCompleted"),
        }
    }
}

impl ReadStreamSuccess {
    #[doc(hidden)]
    pub fn as_message_write<'a>(&'a self) -> ReadStreamEventsCompleted<'a> {
        use client_messages_ext::ResolvedIndexedEventExt;

        ReadStreamEventsCompleted {
            events: self.events.iter().map(|x| x.borrowed()).collect(),
            result: Some(ReadStreamResult::Success),
            next_event_number: self.next_page.map(|x| x.into()).unwrap_or(-1),
            last_event_number: self.last_event_number.into(),
            is_end_of_stream: self.end_of_stream,
            last_commit_position: self.last_commit_position,
            error: None,
        }
    }
}

// NOTE: similar to ReadEventFailure, but this has NotModified instead of NotFound
/// Non-success projection of the `ReadStreamResult` enum on the wire representing
/// a failed `ReadStreamCompleted` request.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ReadStreamFailure {
    /// Stream was not found
    NoStream,
    /// Stream has been deleted
    StreamDeleted,
    /// Unknown when this happens: it is not retuned when reading past the last event in a stream
    /// forwards.
    NotModified,
    /// Other error
    Error(Option<Cow<'static, str>>),
    /// Access was denied (no credentials provided or insufficient permissions)
    AccessDenied,
}

impl Into<(ReadStreamResult, Option<Cow<'static, str>>)> for ReadStreamFailure {
    fn into(self) -> (ReadStreamResult, Option<Cow<'static, str>>) {
        use ReadStreamFailure::*;
        match self {
            NoStream => (ReadStreamResult::NoStream, None),
            StreamDeleted => (ReadStreamResult::StreamDeleted, None),
            NotModified => (ReadStreamResult::NotModified, None),
            Error(x) => (ReadStreamResult::Error, x),
            AccessDenied => (ReadStreamResult::AccessDenied, None),
        }
    }
}

impl<'a> From<(ReadStreamResult, Option<Cow<'a, str>>)> for ReadStreamFailure {
    fn from((res, err): (ReadStreamResult, Option<Cow<'a, str>>)) -> Self {
        use self::ReadStreamResult::*;
        match res {
            Success => unreachable!(),
            NoStream => ReadStreamFailure::NoStream,
            StreamDeleted => ReadStreamFailure::StreamDeleted,
            NotModified => ReadStreamFailure::NotModified,
            Error => ReadStreamFailure::Error(err.map(Cow::into_owned).map(Cow::Owned)),
            AccessDenied => ReadStreamFailure::AccessDenied,
        }
    }
}

impl ReadStreamFailure {
    #[doc(hidden)]
    pub fn as_message_write<'a>(&'a self) -> ReadStreamEventsCompleted<'a> {
        use ReadStreamFailure::*;
        let (res, msg): (ReadStreamResult, Option<Cow<'a, str>>) = match self {
            &NoStream => (ReadStreamResult::NoStream, None),
            &StreamDeleted => (ReadStreamResult::StreamDeleted, None),
            &NotModified => (ReadStreamResult::NotModified, None),
            &Error(ref x) => (ReadStreamResult::Error, match x {
                &Some(ref cow) => Some(Cow::Borrowed(cow)),
                &None => None,
            }),
            &AccessDenied => (ReadStreamResult::AccessDenied, None),
        };

        ReadStreamEventsCompleted {
            events: vec![],
            result: Some(res),
            next_event_number: -1,
            last_event_number: -1,
            is_end_of_stream: false,
            last_commit_position: -1,
            error: msg,
        }
    }
}
