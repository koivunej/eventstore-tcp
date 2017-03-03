use std::borrow::Cow;
use client_messages::{ReadEventCompleted, EventRecord, ResolvedIndexedEvent};
use client_messages::mod_ReadEventCompleted::ReadEventResult;

/// `ReadEventFailure` maps to non-success of `ReadEventResult`
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ReadEventFailure {
    /// Event of requested number was not found (scavenged or never existed)
    NotFound,
    /// No such stream
    NoStream,
    /// Stream has been deleted
    StreamDeleted,
    /// Other error
    Error(Option<Cow<'static, str>>),
    /// Access was denied (no credentials provided or insufficient permissions)
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
    // TODO: this needs to become as_message_write()
    #[doc(hidden)]
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
