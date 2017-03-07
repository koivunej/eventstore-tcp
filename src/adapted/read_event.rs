use std::borrow::Cow;
use raw::client_messages::mod_ReadEventCompleted::ReadEventResult;

/// `ReadEventError` maps to non-success of `ReadEventResult`
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ReadEventError<'a> {
    /// Event of requested number was not found (scavenged or never existed)
    NotFound,
    /// No such stream
    NoStream,
    /// Stream has been deleted
    StreamDeleted,
    /// Other error
    Error(Option<Cow<'a, str>>),
    /// Access was denied (no credentials provided or insufficient permissions)
    AccessDenied
}

impl<'a> From<(ReadEventResult, Option<Cow<'a, str>>)> for ReadEventError<'a> {
    fn from((res, err): (ReadEventResult, Option<Cow<'a, str>>)) -> Self {
        use self::ReadEventResult::*;
        match res {
            Success => unreachable!(),
            NotFound => ReadEventError::NotFound,
            NoStream => ReadEventError::NoStream,
            StreamDeleted => ReadEventError::StreamDeleted,
            Error => ReadEventError::Error(err),
            AccessDenied => ReadEventError::AccessDenied,
        }
    }
}
