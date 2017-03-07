use std::borrow::Cow;
use client_messages::{self, ReadAllEvents, ReadAllEventsCompleted};
use client_messages::mod_ReadAllEventsCompleted::ReadAllResult;

use {Message, LogPosition, ReadDirection};

impl From<(ReadDirection, ReadAllEvents)> for Message {
    fn from((dir, rae): (ReadDirection, ReadAllEvents)) -> Message {
        Message::ReadAllEvents(dir, rae)
    }
}

/// Successful response to `Message::ReadAllEvents`.
#[derive(Debug, Clone, PartialEq)]
pub struct ReadAllSuccess {
    commit_position: LogPosition,
    prepare_position: LogPosition,
    /// The read events, with position metadata
    pub events: Vec<ResolvedEvent<'static>>,
    next_commit_position: Option<LogPosition>,
    next_prepare_position: Option<LogPosition>,
}

/// Successful response to `Message::ReadAllEvents`.
#[derive(Debug, Clone, PartialEq)]
pub struct ReadAllCompleted<'a> {
    pub commit_position: LogPosition,
    pub prepare_position: LogPosition,
    /// The read events, with position metadata
    pub events: Vec<ResolvedEvent<'a>>,
    pub next_commit_position: Option<LogPosition>,
    pub next_prepare_position: Option<LogPosition>,
}

/// Read event in `ReadAllSuccess` response
#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedEvent<'a> {
    /// The read event
    pub event: client_messages::EventRecord<'a>,
    /// Possible linking event
    pub link: Option<client_messages::EventRecord<'a>>,
    /// Position where this events transaction is commited
    pub commit_position: LogPosition,
    /// Position where this event is stored
    pub prepare_position: LogPosition,
}

impl<'a> From<client_messages::ResolvedEvent<'a>> for ResolvedEvent<'a> {
    fn from(e: client_messages::ResolvedEvent<'a>) -> ResolvedEvent<'a> {
        ResolvedEvent {
            event: e.event.into(),
            link: e.link.into(),
            commit_position: e.commit_position.into(),
            prepare_position: e.prepare_position.into()
        }
    }
}

impl<'a> ResolvedEvent<'a> {
    fn into_owned(self) -> ResolvedEvent<'static> {
        use client_messages_ext::EventRecordExt;
        ResolvedEvent {
            event: self.event.into_owned(),
            link: self.link.map(|link| link.into_owned()),
            commit_position: self.commit_position,
            prepare_position: self.prepare_position,
        }
    }

    fn borrowed<'b>(&'b self) -> client_messages::ResolvedEvent<'b> {
        use client_messages_ext::EventRecordExt;
        client_messages::ResolvedEvent {
            event: self.event.borrowed(),
            link: self.link.as_ref().map(|x| x.borrowed()),
            commit_position: self.commit_position.into(),
            prepare_position: self.prepare_position.into(),
        }
    }
}

impl<'a> From<(ReadDirection, ReadAllEventsCompleted<'a>)> for Message {
    fn from((dir, resp): (ReadDirection, ReadAllEventsCompleted<'a>)) -> Message {
        use client_messages::mod_ReadAllEventsCompleted::ReadAllResult;

        let res = match resp.result {
            ReadAllResult::Success => {
                Ok(ReadAllSuccess {
                    commit_position: resp.commit_position.into(),
                    prepare_position: resp.prepare_position.into(),
                    events: resp.events.into_iter().map(|x| {
                        let er: ResolvedEvent<'a> = x.into();
                        er.into_owned()
                    }).collect(),
                    next_commit_position: LogPosition::from_i64_opt(resp.next_commit_position),
                    next_prepare_position: LogPosition::from_i64_opt(resp.next_prepare_position),
                })
            },
            fail => Err((fail, resp.error).into()),
        };

        Message::ReadAllEventsCompleted(dir, res)
    }
}

impl ReadAllSuccess {
    #[doc(hidden)]
    pub fn as_message_write<'a>(&'a self) -> client_messages::ReadAllEventsCompleted<'a> {
        use client_messages::mod_ReadAllEventsCompleted::ReadAllResult;

        client_messages::ReadAllEventsCompleted {
            commit_position: self.commit_position.into(),
            prepare_position: self.prepare_position.into(),
            events: self.events.iter().map(|x| x.borrowed()).collect(),
            next_commit_position: self.next_commit_position.map(|x| x.into()).unwrap_or(-1),
            next_prepare_position: self.next_prepare_position.map(|x| x.into()).unwrap_or(-1),
            result: ReadAllResult::Success,
            error: None,
        }
    }
}

/// Failure cases of wire enum `ReadAllResult`.
#[derive(Debug, Clone, PartialEq)]
pub enum ReadAllFailure {
    /// Unknown when this happens,
    NotModified,
    /// Other error
    Error(Option<Cow<'static, str>>),
    /// Access was denied (no credentials provided or insufficient permissions)
    AccessDenied
}

impl<'a> From<(ReadAllResult, Option<Cow<'a, str>>)> for ReadAllFailure {
    fn from((r, msg): (ReadAllResult, Option<Cow<'a, str>>)) -> ReadAllFailure {
        use self::ReadAllResult::*;
        match r {
            Success => unreachable!(),
            NotModified => ReadAllFailure::NotModified,
            Error => ReadAllFailure::Error(msg.map(|x| Cow::Owned(x.into_owned()))),
            AccessDenied => ReadAllFailure::AccessDenied,
        }
    }
}

impl ReadAllFailure {
    #[doc(hidden)]
    pub fn as_message_write<'a>(&'a self) -> ReadAllEventsCompleted<'a> {
        use ReadAllFailure::*;
        let (res, msg): (ReadAllResult, Option<Cow<'a, str>>) = match self {
            &NotModified => (ReadAllResult::NotModified, None),
            &Error(ref msg) => (ReadAllResult::Error, match msg {
                &Some(ref cow) => Some(Cow::Borrowed(cow)),
                &None => None,
            }),
            &AccessDenied => (ReadAllResult::AccessDenied, None),
        };

        ReadAllEventsCompleted {
            commit_position: -1,
            prepare_position: -1,
            events: vec![],
            next_commit_position: -1,
            next_prepare_position: -1,
            result: res,
            error: msg,
        }
    }
}

/// Failure cases of wire enum `ReadAllResult`.
#[derive(Debug, Clone, PartialEq)]
pub enum ReadAllError<'a> {
    /// Unknown when this happens,
    NotModified,
    /// Other error
    Error(Option<Cow<'a, str>>),
    /// Access was denied (no credentials provided or insufficient permissions)
    AccessDenied
}

impl<'a> From<(ReadAllResult, Option<Cow<'a, str>>)> for ReadAllError<'a> {
    fn from((r, msg): (ReadAllResult, Option<Cow<'a, str>>)) -> ReadAllError<'a> {
        use self::ReadAllResult::*;
        match r {
            Success => unreachable!(),
            NotModified => ReadAllError::NotModified,
            Error => ReadAllError::Error(msg),
            AccessDenied => ReadAllError::AccessDenied,
        }
    }
}
