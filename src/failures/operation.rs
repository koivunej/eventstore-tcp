use std::fmt;
use std::error::Error;
use client_messages::{OperationResult, WriteEventsCompleted};

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

