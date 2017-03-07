use std::borrow::Cow;
use raw::client_messages;
use raw::client_messages::mod_ReadAllEventsCompleted::ReadAllResult;

use LogPosition;

/// Successful response to `Message::ReadAllEvents`.
#[derive(Debug, Clone, PartialEq)]
pub struct ReadAllCompleted<'a> {
    /// Position of the commit of the current prepare
    pub commit_position: LogPosition,
    /// Position of the current prepare
    pub prepare_position: LogPosition,
    /// The read events, with position metadata
    pub events: Vec<ResolvedEvent<'a>>,
    /// For paging: next commit position
    pub next_commit_position: Option<LogPosition>,
    /// For paging: next prepare position
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
