use std::borrow::Cow;
use client_messages::ReadAllEventsCompleted;
use client_messages::mod_ReadAllEventsCompleted::ReadAllResult;

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
