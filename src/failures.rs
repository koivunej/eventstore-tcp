use std::borrow::Cow;
use messages::mod_EventStore::mod_Client::mod_Messages as client_messages;
use messages::mod_EventStore::mod_Client::mod_Messages::{OperationResult, ReadEventCompleted};
use messages::mod_EventStore::mod_Client::mod_Messages::mod_ReadEventCompleted::ReadEventResult;

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

impl OperationFailure {
    pub fn is_transient(&self) -> bool {
        use OperationFailure::*;
        match *self {
            PrepareTimeout | CommitTimeout | ForwardTimeout => true,
            _ => false
        }
    }
}

impl Copy for OperationFailure {}

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
        let (res, msg) = match self {
            &NotFound => (ReadEventResult::NotFound, None),
            &NoStream => (ReadEventResult::NoStream, None),
            &StreamDeleted => (ReadEventResult::StreamDeleted, None),
            &Error(ref x) => (ReadEventResult::Error, match x {
                &Some(ref cow) => Some(Cow::Borrowed(cow)),
                &None => None,
            }),
            &AccessDenied => (ReadEventResult::AccessDenied, None),
        };
        // not sure how event is written here
        unimplemented!()

        /*
        ReadEventCompleted {
            result: res,
            error: msg,
        }
        */
    }
}
