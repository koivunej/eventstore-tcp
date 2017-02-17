use messages::mod_EventStore::mod_Client::mod_Messages as client_messages;

/// Like `OperationResult` on the wire but does not have a success value. Explains the reason for
/// failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OperationResult {
    PrepareTimeout,
    CommitTimeout,
    ForwardTimeout,
    WrongExpectedVersion,
    StreamDeleted,
    InvalidTransaction,
    AccessDenied,
}

impl Copy for OperationResult {}

impl From<client_messages::OperationResult> for OperationResult {
    fn from(or: client_messages::OperationResult) -> OperationResult {
        use client_messages::OperationResult::*;

        match or {
            Success => panic!("Success type is invalid for this type"),
            PrepareTimeout => OperationResult::PrepareTimeout,
            CommitTimeout => OperationResult::CommitTimeout,
            ForwardTimeout => OperationResult::ForwardTimeout,
            WrongExpectedVersion => OperationResult::WrongExpectedVersion,
            StreamDeleted => OperationResult::StreamDeleted,
            InvalidTransaction => OperationResult::InvalidTransaction,
            AccessDenied => OperationResult::AccessDenied,
        }
    }
}

impl Into<client_messages::OperationResult> for OperationResult {
    fn into(self) -> client_messages::OperationResult {
        use OperationResult::*;
        match self {
            PrepareTimeout => client_messages::OperationResult::PrepareTimeout,
            CommitTimeout => client_messages::OperationResult::CommitTimeout,
            ForwardTimeout => client_messages::OperationResult::ForwardTimeout,
            WrongExpectedVersion => client_messages::OperationResult::WrongExpectedVersion,
            StreamDeleted => client_messages::OperationResult::StreamDeleted,
            InvalidTransaction => client_messages::OperationResult::InvalidTransaction,
            AccessDenied => client_messages::OperationResult::AccessDenied
        }
    }
}


