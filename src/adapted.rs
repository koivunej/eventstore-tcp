use std::borrow::Cow;
use {CustomTryFrom, CustomTryInto, ResultStatusKind, MappingErrorKind, MappingError, ReadDirection};
use client_messages::{self, WriteEvents};
use client_messages::mod_NotHandled::{NotHandledReason, MasterInfo};
use raw;
use write_events::{WriteEventsCompleted, WriteEventsFailure};
use read_event::{ReadEventFailure};
use read_stream::{ReadStreamSuccess, ReadStreamFailure};
use read_all::{ReadAllSuccess, ReadAllFailure};

enum AdaptedMessage<'a> {
    /// Requests heartbeat from the other side. Unsure if clients or server sends these.
    HeartbeatRequest,
    /// Response to a heartbeat request.
    HeartbeatResponse,

    /// Ping request, similar to heartbeat.
    Ping,
    /// Ping response.
    Pong,

    /// Append to stream request
    WriteEvents(WriteEvents<'a>),
    /// Append to stream response, which can fail for a number of reasons
    WriteEventsCompleted(Result<WriteEventsCompleted, WriteEventsFailure>),

    /// Request to read a single event from a stream
    ReadEvent(client_messages::ReadEvent<'a>),
    /// Response to a single event read
    ReadEventCompleted(Result<client_messages::ResolvedIndexedEvent<'a>, ReadEventFailure>),

    /// Request to read a stream from a point forward or backward
    ReadStreamEvents(ReadDirection, client_messages::ReadStreamEvents<'a>),
    /// Response to a stream read in given direction
    ReadStreamEventsCompleted(ReadDirection, Result<ReadStreamSuccess, ReadStreamFailure>),

    /// Request to read a stream of all events from a position forward or backward
    ReadAllEvents(ReadDirection, client_messages::ReadAllEvents),
    /// Response to a read all in given direction
    ReadAllEventsCompleted(ReadDirection, Result<ReadAllSuccess, ReadAllFailure>),

    /// Request was not understood. Please open an issue!
    BadRequest(BadRequestMessage<'a>),

    /// Correlated request was not handled. This is the likely response to requests where
    /// `require_master` is `true`, but the connected endpoint is not master and cannot reach it.
    NotHandled(NotHandledReason, Option<MasterInfo<'a>>),

    /// Request to authenticate attached credentials.
    Authenticate,

    /// Positive authentication response. The credentials used to `Authenticate` previously can be
    /// used in successive requests.
    Authenticated,

    /// Negative authentication response, or response to any sent request for which used
    /// authentication was not accepted.
    NotAuthenticated(NotAuthenticatedMessage<'a>)
}

impl<'a> CustomTryFrom<raw::RawMessage<'a>> for AdaptedMessage<'a> {
    type Err = MappingError;
    fn try_from(raw: raw::RawMessage<'a>) -> Result<AdaptedMessage<'a>, (raw::RawMessage<'a>, MappingError)> {
        use raw::{RawMessage, ByteWrapper};

        macro_rules! into_or_rebuild {
            ($x: expr) => {
                {
                    Ok($x.try_into().map_err(|(x, e)| {
                        let orig = raw::RawMessage::from(x);
                        let err = MappingError::from(e);
                        (orig, err)
                    })?)
                }
            }
        }

        macro_rules! into_str_or_rebuild {
            ($x: expr, $ctor: expr) => {
                {
                    let res: Result<Cow<'a, str>, (raw::RawMessage<'a>, MappingError)> =
                        $x.into_str_wrapper().map_err(|(wrapper, e)| (wrapper.into(), e));

                    let cow_str: Cow<'a, str> = res?;
                    Ok($ctor(cow_str).into())
                }
            }
        }

        match raw {
            RawMessage::HeartbeatRequest                  => Ok(AdaptedMessage::HeartbeatRequest),
            RawMessage::HeartbeatResponse                 => Ok(AdaptedMessage::HeartbeatResponse),
            RawMessage::Ping                              => Ok(AdaptedMessage::Ping),
            RawMessage::Pong                              => Ok(AdaptedMessage::Pong),
            RawMessage::WriteEvents(e)                    => into_or_rebuild!(e),
            RawMessage::WriteEventsCompleted(e)           => into_or_rebuild!(e),
            RawMessage::ReadEvent(e)                      => into_or_rebuild!(e),
            RawMessage::ReadEventCompleted(e)             => into_or_rebuild!(e),
            RawMessage::ReadStreamEvents(dir, e)          => into_or_rebuild!((dir, e)),
            RawMessage::ReadStreamEventsCompleted(dir, e) => into_or_rebuild!((dir, e)),
            RawMessage::ReadAllEvents(dir, e)             => into_or_rebuild!((dir, e)),
            RawMessage::ReadAllEventsCompleted(dir, e)    => into_or_rebuild!((dir, e)),
            RawMessage::BadRequest(bytes)                 => into_str_or_rebuild!(bytes, BadRequestMessage::from),
            RawMessage::NotHandled(e)                     => into_or_rebuild!(e),
            RawMessage::Authenticate                      => Ok(AdaptedMessage::Authenticate),
            RawMessage::Authenticated                     => Ok(AdaptedMessage::Authenticated),
            RawMessage::NotAuthenticated(reason)          => into_str_or_rebuild!(reason, NotAuthenticatedMessage::from),
            RawMessage::Unsupported(d, bytes)             => Err(((d, bytes).into(), MappingErrorKind::Unsupported(d).into())),
            _ => unimplemented!()
        }
    }
}

/// Newtype for wrapping a specific message, AdaptedMessage::BadRequest
struct BadRequestMessage<'a>(Cow<'a, str>);

impl<'a> AsRef<str> for BadRequestMessage<'a> {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl<'a> From<Cow<'a, str>> for BadRequestMessage<'a> {
    fn from(s: Cow<'a, str>) -> BadRequestMessage<'a> {
        BadRequestMessage(s)
    }
}

impl<'a> Into<Cow<'a, str>> for BadRequestMessage<'a> {
    fn into(self) -> Cow<'a, str> {
        self.0
    }
}

impl<'a> From<BadRequestMessage<'a>> for AdaptedMessage<'a> {
    fn from(m: BadRequestMessage<'a>) -> AdaptedMessage<'a> {
        AdaptedMessage::BadRequest(m)
    }
}

/// Newtype for wrapping a specific message, AdaptedMessage::NotAuthenticated
struct NotAuthenticatedMessage<'a>(Cow<'a, str>);

impl<'a> AsRef<str> for NotAuthenticatedMessage<'a> {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl<'a> From<Cow<'a, str>> for NotAuthenticatedMessage<'a> {
    fn from(s: Cow<'a, str>) -> NotAuthenticatedMessage<'a> {
        NotAuthenticatedMessage(s)
    }
}

impl<'a> Into<Cow<'a, str>> for NotAuthenticatedMessage<'a> {
    fn into(self) -> Cow<'a, str> {
        self.0
    }
}

impl<'a> From<NotAuthenticatedMessage<'a>> for AdaptedMessage<'a> {
    fn from(m: NotAuthenticatedMessage<'a>) -> AdaptedMessage<'a> {
        AdaptedMessage::NotAuthenticated(m)
    }
}

impl<'a> CustomTryFrom<client_messages::NotHandled<'a>> for AdaptedMessage<'a> {
    type Err = MappingError;

    fn try_from(msg: client_messages::NotHandled<'a>) -> Result<AdaptedMessage<'a>, (client_messages::NotHandled<'a>, Self::Err)> {
        unimplemented!()
    }
}

impl<'a> CustomTryFrom<client_messages::WriteEvents<'a>> for AdaptedMessage<'a> {
    type Err = MappingError;

    fn try_from(msg: client_messages::WriteEvents<'a>) -> Result<AdaptedMessage<'a>, (client_messages::WriteEvents<'a>, Self::Err)> {
        Ok(AdaptedMessage::WriteEvents(msg))
    }
}

impl<'a> CustomTryFrom<client_messages::WriteEventsCompleted<'a>> for AdaptedMessage<'a> {
    type Err = MappingError;

    fn try_from(msg: client_messages::WriteEventsCompleted<'a>) -> Result<AdaptedMessage<'a>, (client_messages::WriteEventsCompleted<'a>, Self::Err)> {
        unimplemented!()
    }
}

impl<'a> CustomTryFrom<client_messages::ReadEvent<'a>> for AdaptedMessage<'a> {
    type Err = MappingError;

    fn try_from(msg: client_messages::ReadEvent<'a>) -> Result<AdaptedMessage<'a>, (client_messages::ReadEvent<'a>, Self::Err)> {
        unimplemented!()
    }
}

impl<'a> CustomTryFrom<client_messages::ReadEventCompleted<'a>> for AdaptedMessage<'a> {
    type Err = MappingError;

    fn try_from(msg: client_messages::ReadEventCompleted<'a>) -> Result<AdaptedMessage<'a>, (client_messages::ReadEventCompleted<'a>, Self::Err)> {
        unimplemented!()
    }
}

impl<'a> CustomTryFrom<(ReadDirection, client_messages::ReadStreamEvents<'a>)> for AdaptedMessage<'a> {
    type Err = MappingError;

    fn try_from((dir, msg): (ReadDirection, client_messages::ReadStreamEvents<'a>)) -> Result<AdaptedMessage<'a>, ((ReadDirection, client_messages::ReadStreamEvents<'a>), Self::Err)> {
        unimplemented!()
    }
}

impl<'a> CustomTryFrom<(ReadDirection, client_messages::ReadStreamEventsCompleted<'a>)> for AdaptedMessage<'a> {
    type Err = MappingError;

    fn try_from((dir, msg): (ReadDirection, client_messages::ReadStreamEventsCompleted<'a>)) -> Result<AdaptedMessage<'a>, ((ReadDirection, client_messages::ReadStreamEventsCompleted<'a>), Self::Err)> {
        unimplemented!()
    }
}

impl<'a> CustomTryFrom<(ReadDirection, client_messages::ReadAllEvents)> for AdaptedMessage<'a> {
    type Err = MappingError;

    fn try_from((dir, msg): (ReadDirection, client_messages::ReadAllEvents)) -> Result<AdaptedMessage<'a>, ((ReadDirection, client_messages::ReadAllEvents), Self::Err)> {
        unimplemented!()
    }
}

impl<'a> CustomTryFrom<(ReadDirection, client_messages::ReadAllEventsCompleted<'a>)> for AdaptedMessage<'a> {
    type Err = MappingError;

    fn try_from((dir, msg): (ReadDirection, client_messages::ReadAllEventsCompleted<'a>)) -> Result<AdaptedMessage<'a>, ((ReadDirection, client_messages::ReadAllEventsCompleted<'a>), Self::Err)> {
        unimplemented!()
    }
}
