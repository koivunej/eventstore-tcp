use std::fmt;
use std::error::Error;
use client_messages::{OperationResult, WriteEventsCompleted};
use AsMessageWrite;

/// Like `OperationResult` on the wire but does not have a success value. Explains the reason for
/// failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WriteEventsFailure {
    /// Server failed to process the request before timeout
    PrepareTimeout,
    /// Server timed out while awaiting commit to be processed
    CommitTimeout,
    /// Server timed out while awaiting for a forwarded request to complete
    ForwardTimeout,
    /// Optimistic locking failure; stream version was not the expected
    WrongExpectedVersion,
    /// Stream has been deleted
    StreamDeleted,
    /// No authentication provided or insufficient permissions to a stream
    AccessDenied,
}

impl Copy for WriteEventsFailure {}

impl WriteEventsFailure {
    /// Return `true` if the operation failed in a transient way that might be resolved by
    /// retrying.
    pub fn is_transient(&self) -> bool {
        use WriteEventsFailure::*;
        match *self {
            PrepareTimeout | CommitTimeout | ForwardTimeout => true,
            _ => false
        }
    }
}

impl AsMessageWrite<WriteEventsCompleted<'static>> for WriteEventsFailure {
    fn as_message_write(&self) -> WriteEventsCompleted<'static> {
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

impl From<OperationResult> for WriteEventsFailure {
    fn from(or: OperationResult) -> Self {
        use self::OperationResult::*;

        match or {
            Success => unreachable!(),
            InvalidTransaction => unreachable!(),
            PrepareTimeout => WriteEventsFailure::PrepareTimeout,
            CommitTimeout => WriteEventsFailure::CommitTimeout,
            ForwardTimeout => WriteEventsFailure::ForwardTimeout,
            WrongExpectedVersion => WriteEventsFailure::WrongExpectedVersion,
            StreamDeleted => WriteEventsFailure::StreamDeleted,
            AccessDenied => WriteEventsFailure::AccessDenied,
        }
    }
}

impl Into<OperationResult> for WriteEventsFailure {
    fn into(self) -> OperationResult {
        use WriteEventsFailure::*;
        match self {
            PrepareTimeout => OperationResult::PrepareTimeout,
            CommitTimeout => OperationResult::CommitTimeout,
            ForwardTimeout => OperationResult::ForwardTimeout,
            WrongExpectedVersion => OperationResult::WrongExpectedVersion,
            StreamDeleted => OperationResult::StreamDeleted,
            AccessDenied => OperationResult::AccessDenied
        }
    }
}

impl fmt::Display for WriteEventsFailure {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.description())
    }
}

impl Error for WriteEventsFailure {
    fn description(&self) -> &str {
        use WriteEventsFailure::*;
        match *self {
            PrepareTimeout => "Internal server timeout, should be retried",
            CommitTimeout => "Internal server timeout, should be retried",
            ForwardTimeout => "Server timed out while awaiting response to forwarded request, should be retried",
            WrongExpectedVersion => "Stream version was not expected, optimistic locking failure",
            StreamDeleted => "Stream had been deleted",
            AccessDenied => "Access to stream was denied"
        }
    }
}

