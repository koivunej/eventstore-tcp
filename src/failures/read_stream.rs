use std::borrow::Cow;
use client_messages::ReadStreamEventsCompleted;
use client_messages::mod_ReadStreamEventsCompleted::ReadStreamResult;

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
