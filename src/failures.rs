use std::fmt;
use std::error::Error;
use std::borrow::Cow;
use client_messages::{OperationResult, WriteEventsCompleted, ReadEventCompleted, ReadStreamEventsCompleted, EventRecord, ResolvedIndexedEvent};
use client_messages::mod_ReadEventCompleted::ReadEventResult;
use client_messages::mod_ReadStreamEventsCompleted::ReadStreamResult;

/// Like `OperationResult` on the wire but does not have a success value. Explains the reason for
/// failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OperationFailure {
    PrepareTimeout,
    CommitTimeout,
    ForwardTimeout,
    WrongExpectedVersion,
    StreamDeleted,
    InvalidTransaction,
    AccessDenied,
}

impl Copy for OperationFailure {}

impl OperationFailure {
    pub fn is_transient(&self) -> bool {
        use OperationFailure::*;
        match *self {
            PrepareTimeout | CommitTimeout | ForwardTimeout => true,
            _ => false
        }
    }

    pub fn as_write_events_completed<'a>(&'a self) -> WriteEventsCompleted<'a> {
        WriteEventsCompleted {
            result: Some(self.clone().into()),
            message: None,
            first_event_number: -1,
            last_event_number: -1,
            prepare_position: None,
            commit_position: None,
        }
    }
}

impl From<OperationResult> for OperationFailure {
    fn from(or: OperationResult) -> Self {
        use self::OperationResult::*;

        match or {
            Success => unreachable!(),
            PrepareTimeout => OperationFailure::PrepareTimeout,
            CommitTimeout => OperationFailure::CommitTimeout,
            ForwardTimeout => OperationFailure::ForwardTimeout,
            WrongExpectedVersion => OperationFailure::WrongExpectedVersion,
            StreamDeleted => OperationFailure::StreamDeleted,
            InvalidTransaction => OperationFailure::InvalidTransaction,
            AccessDenied => OperationFailure::AccessDenied,
        }
    }
}

impl Into<OperationResult> for OperationFailure {
    fn into(self) -> OperationResult {
        use OperationFailure::*;
        match self {
            PrepareTimeout => OperationResult::PrepareTimeout,
            CommitTimeout => OperationResult::CommitTimeout,
            ForwardTimeout => OperationResult::ForwardTimeout,
            WrongExpectedVersion => OperationResult::WrongExpectedVersion,
            StreamDeleted => OperationResult::StreamDeleted,
            InvalidTransaction => OperationResult::InvalidTransaction,
            AccessDenied => OperationResult::AccessDenied
        }
    }
}

impl fmt::Display for OperationFailure {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.description())
    }
}

impl Error for OperationFailure {
    fn description(&self) -> &str {
        use OperationFailure::*;
        match *self {
            PrepareTimeout => "Internal server timeout, should be retried",
            CommitTimeout => "Internal server timeout, should be retried",
            ForwardTimeout => "Server timed out while awaiting response to forwarded request, should be retried",
            WrongExpectedVersion => "Stream version was not expected, optimistic locking failure",
            StreamDeleted => "Stream had been deleted",
            InvalidTransaction => "Transaction had been rolled back",
            AccessDenied => "Access to stream was denied"
        }
    }
}

/// `ReadEventFailure` maps to non-success of `ReadEventResult`
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ReadEventFailure {
    NotFound,
    NoStream,
    StreamDeleted,
    Error(Option<Cow<'static, str>>),
    AccessDenied
}

impl<'a> From<(ReadEventResult, Option<Cow<'a, str>>)> for ReadEventFailure {
    fn from((res, err): (ReadEventResult, Option<Cow<'a, str>>)) -> Self {
        use self::ReadEventResult::*;
        match res {
            Success => unreachable!(),
            NotFound => ReadEventFailure::NotFound,
            NoStream => ReadEventFailure::NoStream,
            StreamDeleted => ReadEventFailure::StreamDeleted,
            Error => ReadEventFailure::Error(err.map(Cow::into_owned).map(Cow::Owned)),
            AccessDenied => ReadEventFailure::AccessDenied,
        }
    }
}

impl Into<(ReadEventResult, Option<Cow<'static, str>>)> for ReadEventFailure {
    fn into(self) -> (ReadEventResult, Option<Cow<'static, str>>) {
        use ReadEventFailure::*;
        match self {
            NotFound => (ReadEventResult::NotFound, None),
            NoStream => (ReadEventResult::NoStream, None),
            StreamDeleted => (ReadEventResult::StreamDeleted, None),
            Error(x) => (ReadEventResult::Error, x),
            AccessDenied => (ReadEventResult::AccessDenied, None),
        }
    }
}

impl ReadEventFailure {
    pub fn as_read_event_completed<'a>(&'a self) -> ReadEventCompleted<'a> {
        use ReadEventFailure::*;
        let (res, msg): (ReadEventResult, Option<Cow<'a, str>>) = match self {
            &NotFound => (ReadEventResult::NotFound, None),
            &NoStream => (ReadEventResult::NoStream, None),
            &StreamDeleted => (ReadEventResult::StreamDeleted, None),
            &Error(ref x) => (ReadEventResult::Error, match x {
                &Some(ref cow) => Some(Cow::Borrowed(&*cow)),
                &None => None,
            }),
            &AccessDenied => (ReadEventResult::AccessDenied, None),
        };

        ReadEventCompleted {
            result: Some(res),
            event: ResolvedIndexedEvent {
                event: EventRecord {
                    event_stream_id: "".into(),
                    event_number: -1,
                    event_id: Cow::Borrowed(&[]),
                    event_type: "".into(),
                    data_content_type: 0,
                    metadata_content_type: 0,
                    data: Cow::Borrowed(&[]),
                    metadata: None,
                    created: None,
                    created_epoch: None,
                },
                link: None,
            },
            error: msg,
        }
    }

}

// NOTE: similar to ReadEventFailure, but this has NotModified instead of NotFound
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ReadStreamFailure {
    NoStream,
    StreamDeleted,
    NotModified,
    Error(Option<Cow<'static, str>>),
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
    pub fn as_read_stream_events_completed<'a>(&'a self) -> ReadStreamEventsCompleted<'a> {
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
