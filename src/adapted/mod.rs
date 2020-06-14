//! Adapted or refined types providing a much more oxidized API for handling the messages in the
//! protocol.

use std::convert::{TryFrom, TryInto};
use std::borrow::Cow;
use std::ops::Range;
use errors::{Error, ErrorKind, ResultStatusKind};
use crate::{ReadDirection, EventNumber, StreamVersion, LogPosition};
use raw;
use raw::client_messages::{WriteEvents, ResolvedIndexedEvent};

mod write_events;
pub use self::write_events::{WriteEventsCompleted, WriteEventsFailure};

mod read_event;
pub use self::read_event::{ReadEventError};

mod read_stream;
pub use self::read_stream::{ReadStreamCompleted, ReadStreamError};

mod read_all;
pub use self::read_all::{ReadAllCompleted, ReadAllError};

/// Enumeration of converted messages for more oxidized API. Unlike the `RawMessage` variants,
/// `AdaptedMessage` variants are validated and converted into nicer API. This validation comes at
/// a cost of a fallible conversion exposed in `TryFrom` implementation.
#[derive(Debug, PartialEq, Clone)]
pub enum AdaptedMessage<'a> {
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
    ReadEvent(raw::client_messages::ReadEvent<'a>),
    /// Response to a single event read
    ReadEventCompleted(Result<raw::client_messages::ResolvedIndexedEvent<'a>, ReadEventError<'a>>),

    /// Request to read a stream from a point forward or backward
    ReadStreamEvents(ReadDirection, raw::client_messages::ReadStreamEvents<'a>),
    /// Response to a stream read in given direction
    ReadStreamEventsCompleted(ReadDirection, Result<ReadStreamCompleted<'a>, ReadStreamError<'a>>),

    /// Request to read a stream of all events from a position forward or backward
    ReadAllEvents(ReadDirection, raw::client_messages::ReadAllEvents),
    /// Response to a read all in given direction
    ReadAllEventsCompleted(ReadDirection, Result<ReadAllCompleted<'a>, ReadAllError<'a>>),

    /// Request was not understood. Please open an issue!
    BadRequest(BadRequestMessage<'a>),

    /// Correlated request was not handled. This is the likely response to requests where
    /// `require_master` is `true`, but the connected endpoint is not master and cannot reach it.
    NotHandled(NotHandledInfo<'a>),

    /// Request to authenticate attached credentials.
    Authenticate,

    /// Positive authentication response. The credentials used to `Authenticate` previously can be
    /// used in successive requests.
    Authenticated,

    /// Negative authentication response, or response to any sent request for which used
    /// authentication was not accepted.
    NotAuthenticated(NotAuthenticatedMessage<'a>)
}

// NOTE: While I still think this was in general a good idea to layer more idiomatic layers above
// the message layers just the implementation is horrible
impl<'a> TryFrom<raw::RawMessage<'a>> for AdaptedMessage<'a> {
    type Error = (raw::RawMessage<'a>, Error);
    fn try_from(raw: raw::RawMessage<'a>) -> Result<AdaptedMessage<'a>, Self::Error> {
        use raw::{RawMessage, ByteWrapper};

        macro_rules! into_or_rebuild {
            ($x: expr) => {
                {
                    let res: Result<AdaptedMessage<'_>, _> = AdaptedMessage::try_from($x);

                    match res {
                        Ok(adapted) => Ok(adapted),
                        Err((x, e)) => {
                            let orig = raw::RawMessage::from(x);
                            let err = Error::from(e);
                            return Err((orig, err))
                        }
                    }
                }
            }
        }

        macro_rules! into_str_or_rebuild {
            ($x: expr, $ctor: expr) => {
                {
                    let res: Result<Cow<'a, str>, (raw::RawMessage<'a>, Error)> =
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
            RawMessage::WriteEvents(e)                    => Ok(AdaptedMessage::from(e)),
            RawMessage::WriteEventsCompleted(e)           => into_or_rebuild!(e),
            RawMessage::ReadEvent(e)                      => Ok(e.into()),
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
            RawMessage::Unsupported(d, bytes)             => Err(((d, bytes).into(), ErrorKind::UnsupportedDiscriminator(d).into())),
            unimpl => Err((unimpl, ErrorKind::UnimplementedConversion.into())),
        }
    }
}

impl<'a> AdaptedMessage<'a> {
    /// Converts the adapted back to raw for encoding.
    pub fn as_raw<'b>(&'b self) -> raw::RawMessage<'b> {
        use self::AdaptedMessage::*;
        use raw::RawMessage;
        match *self {
            HeartbeatRequest => RawMessage::HeartbeatRequest,
            HeartbeatResponse => RawMessage::HeartbeatResponse,
            Ping => RawMessage::Ping,
            Pong => RawMessage::Pong,
            Authenticate => RawMessage::Authenticate,
            Authenticated => RawMessage::Authenticated,
            WriteEvents(ref we) => RawMessage::WriteEvents(we.clone()),
            WriteEventsCompleted(Ok(ref body)) => RawMessage::WriteEventsCompleted(body.as_raw()),
            WriteEventsCompleted(Err(ref err)) => RawMessage::WriteEventsCompleted(err.as_raw()),
            ReadEvent(ref re) => RawMessage::ReadEvent(re.clone()),
            ReadEventCompleted(Ok(ref event)) => RawMessage::ReadEventCompleted(event.as_raw()),
            ReadEventCompleted(Err(ref err)) => RawMessage::ReadEventCompleted(err.as_raw()),
            ReadStreamEvents(ref dir, ref rse) => RawMessage::ReadStreamEvents(*dir, rse.clone()),
            ReadStreamEventsCompleted(ref dir, Ok(ref body)) => RawMessage::ReadStreamEventsCompleted(*dir, body.as_raw()),
            ReadStreamEventsCompleted(ref dir, Err(ref err)) => RawMessage::ReadStreamEventsCompleted(*dir, err.as_raw()),
            ReadAllEvents(ref dir, ref rae) => RawMessage::ReadAllEvents(*dir, rae.clone()),
            ReadAllEventsCompleted(ref dir, Ok(ref body)) => RawMessage::ReadAllEventsCompleted(*dir, body.as_raw()),
            ReadAllEventsCompleted(ref dir, Err(ref err)) => RawMessage::ReadAllEventsCompleted(*dir, err.as_raw()),
            BadRequest(ref msg) => RawMessage::BadRequest(msg.as_raw()),
            NotHandled(ref info) => RawMessage::NotHandled(info.as_raw()),
            NotAuthenticated(ref msg) => RawMessage::NotAuthenticated(msg.as_raw()),
        }
    }
}

trait AsRawPayload<'a, 'b, P: 'b> {
    fn as_raw(&'b self) -> P;
}

/// Placeholder for NotHandledInfo, unused.
#[derive(Debug, PartialEq, Clone)]
pub struct NotHandledInfo<'a> {
    ph: &'a str
}

impl<'a, 'b: 'a> AsRawPayload<'a, 'b, raw::client_messages::NotHandled<'b>> for NotHandledInfo<'a> {
    fn as_raw(&'b self) -> raw::client_messages::NotHandled<'b> {
        unimplemented!()
    }
}

/// Newtype for wrapping a specific message, AdaptedMessage::BadRequest
#[derive(Debug, PartialEq, Clone, From, Into)]
pub struct BadRequestMessage<'a>(Cow<'a, str>);

impl<'a, 'b: 'a> AsRawPayload<'a, 'b, raw::BadRequestPayload<'b>> for BadRequestMessage<'a> {
    fn as_raw(&'b self) -> raw::BadRequestPayload<'b> {
        match self.0 {
            Cow::Owned(ref s) => Cow::Borrowed(s.as_bytes()),
            Cow::Borrowed(ref s) => Cow::Borrowed(s.as_bytes()),
        }.into()
    }
}

impl<'a> AsRef<str> for BadRequestMessage<'a> {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl<'a> From<BadRequestMessage<'a>> for AdaptedMessage<'a> {
    fn from(b: BadRequestMessage<'a>) -> Self {
        AdaptedMessage::BadRequest(b)
    }
}

/// Newtype for wrapping a specific message, AdaptedMessage::NotAuthenticated
#[derive(Debug, PartialEq, Clone, From, Into)]
pub struct NotAuthenticatedMessage<'a>(Cow<'a, str>);

impl<'a, 'b: 'a> AsRawPayload<'a, 'b, raw::NotAuthenticatedPayload<'b>> for NotAuthenticatedMessage<'a> {
    fn as_raw(&'b self) -> raw::NotAuthenticatedPayload<'b> {
        match self.0 {
            Cow::Owned(ref s) => Cow::Borrowed(s.as_bytes()),
            Cow::Borrowed(ref s) => Cow::Borrowed(s.as_bytes()),
        }.into()
    }
}

impl<'a> AsRef<str> for NotAuthenticatedMessage<'a> {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl<'a> From<NotAuthenticatedMessage<'a>> for AdaptedMessage<'a> {
    fn from(na: NotAuthenticatedMessage<'a>) -> Self {
        AdaptedMessage::NotAuthenticated(na)
    }
}

impl<'a> TryFrom<raw::client_messages::NotHandled<'a>> for AdaptedMessage<'a> {
    type Error = (raw::client_messages::NotHandled<'a>, Error);

    fn try_from(msg: raw::client_messages::NotHandled<'a>) -> Result<AdaptedMessage<'a>, Self::Error> {
        Err((msg, ErrorKind::UnimplementedConversion.into()))
    }
}

impl<'a> From<raw::client_messages::WriteEvents<'a>> for AdaptedMessage<'a> {
    fn from(msg: raw::client_messages::WriteEvents<'a>) -> AdaptedMessage<'a> {
        AdaptedMessage::WriteEvents(msg)
    }
}

fn range_from_parts(start: i32, end: i32) -> Result<Range<StreamVersion>, Error> {
    let start = StreamVersion::try_from(start)?;
    let end = StreamVersion::try_from(end + 1)?;
    Ok(start..end)
}

fn range_to_parts(r: &Range<StreamVersion>) -> (i32, i32) {
    let start: i32 = r.start.into();
    let end: i32 = r.end.into();
    (start, end - 1)
}

impl<'a> TryFrom<raw::client_messages::WriteEventsCompleted<'a>> for AdaptedMessage<'a> {
    type Error = (raw::client_messages::WriteEventsCompleted<'a>, Error);

    fn try_from(msg: raw::client_messages::WriteEventsCompleted<'a>) -> Result<AdaptedMessage<'a>, Self::Error> {
        use raw::client_messages::OperationResult::*;

        if !msg.result.is_some() {
            return Err((msg, ErrorKind::MissingResultField(ResultStatusKind::WriteEvents).into()));
        }

        let status = msg.result.clone().unwrap();

        let res = match status {
            Success => {
                let range = match range_from_parts(msg.first_event_number, msg.last_event_number) {
                                Ok(x) => x,
                                Err(e) => return Err( (msg, e) ),
                            };
                Ok(WriteEventsCompleted {
                    event_numbers: range,
                    // unsure if these should be:
                    //  * separate (instead of newtype for tuple)
                    // these must be options, as for idempotent writes the positions might not be
                    // returned. both seem to be returned always.
                    prepare_position: msg.prepare_position.map(|x| x.try_into().unwrap()),
                    commit_position: msg.commit_position.map(|x| x.try_into().unwrap()),
                })
            }
            InvalidTransaction => {
                // this is likely the wrong place to guard this, but changing would require
                // using WriteEventsError::TryFrom
                return Err((msg, ErrorKind::WriteEventsInvalidTransaction.into()));
            }
            other => Err(other.into()),
        };

        Ok(AdaptedMessage::WriteEventsCompleted(res))
    }
}

impl<'b> AsRawPayload<'static, 'b, raw::client_messages::WriteEventsCompleted<'b>> for WriteEventsCompleted {
    fn as_raw(&'b self) -> raw::client_messages::WriteEventsCompleted<'b> {
        let parts = range_to_parts(&self.event_numbers);
        raw::client_messages::WriteEventsCompleted {
            result: Some(raw::client_messages::OperationResult::Success),
            message: None,
            first_event_number: parts.0,
            last_event_number: parts.1,
            prepare_position: self.prepare_position.map(|x| x.into()),
            commit_position: self.commit_position.map(|x| x.into()),
        }
    }
}

impl<'b> AsRawPayload<'static, 'b, raw::client_messages::WriteEventsCompleted<'b>> for WriteEventsFailure {
    fn as_raw(&'b self) -> raw::client_messages::WriteEventsCompleted<'b> {
        raw::client_messages::WriteEventsCompleted {
            result: Some((*self).into()),
            message: None,
            first_event_number: -1,
            last_event_number: -1,
            prepare_position: None,
            commit_position: None,
        }
    }
}

impl<'a> From<raw::client_messages::ReadEvent<'a>> for AdaptedMessage<'a> {
    fn from(msg: raw::client_messages::ReadEvent<'a>) -> AdaptedMessage<'a> {
        // TODO: could map event_number into EventNumber
        AdaptedMessage::ReadEvent(msg)
    }
}

impl<'a> TryFrom<raw::client_messages::ReadEventCompleted<'a>> for AdaptedMessage<'a> {
    type Error = (raw::client_messages::ReadEventCompleted<'a>, Error);

    fn try_from(msg: raw::client_messages::ReadEventCompleted<'a>) -> Result<AdaptedMessage<'a>, Self::Error> {
        use raw::client_messages::mod_ReadEventCompleted::ReadEventResult;

        if msg.result.is_none() {
            return Err((
                    msg,
                    ErrorKind::MissingResultField(ResultStatusKind::ReadEvent).into()));
        }

        match msg.result.unwrap() {
            // TODO: use own type here to have better ResolvedEvent
            // TODO: own ResolvedEvent
            ReadEventResult::Success => Ok(AdaptedMessage::ReadEventCompleted(Ok(msg.event))),
            other => Ok(AdaptedMessage::ReadEventCompleted(Err((other, msg.error).into()))),
        }
    }
}

impl<'a, 'b: 'a> AsRawPayload<'a, 'b, raw::client_messages::ReadEventCompleted<'b>> for ResolvedIndexedEvent<'a> {
    fn as_raw(&'b self) -> raw::client_messages::ReadEventCompleted<'b> {
        raw::client_messages::ReadEventCompleted {
            result: Some(raw::client_messages::mod_ReadEventCompleted::ReadEventResult::Success),
            event: self.clone(),
            error: None,
        }
    }
}

impl<'a, 'b: 'a> AsRawPayload<'a, 'b, raw::client_messages::ReadEventCompleted<'b>> for ReadEventError<'a> {
    fn as_raw(&'b self) -> raw::client_messages::ReadEventCompleted<'b> {
        use self::ReadEventError::*;
        use raw::client_messages::EventRecord;
        use raw::client_messages::mod_ReadEventCompleted::ReadEventResult;

        let (res, msg): (ReadEventResult, Option<Cow<'b, str>>) = match self {
            &NotFound => (ReadEventResult::NotFound, None),
            &NoStream => (ReadEventResult::NoStream, None),
            &StreamDeleted => (ReadEventResult::StreamDeleted, None),
            &Error(ref x) => (ReadEventResult::Error, match x {
                &Some(ref cow) => Some(Cow::Borrowed(&*cow)),
                &None => None,
            }),
            &AccessDenied => (ReadEventResult::AccessDenied, None),
        };

        raw::client_messages::ReadEventCompleted {
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
            error: msg
        }
    }
}

impl<'a> TryFrom<(ReadDirection, raw::client_messages::ReadStreamEvents<'a>)> for AdaptedMessage<'a> {
    type Error = ((ReadDirection, raw::client_messages::ReadStreamEvents<'a>), Error);

    fn try_from((dir, msg): (ReadDirection, raw::client_messages::ReadStreamEvents<'a>)) -> Result<AdaptedMessage<'a>, Self::Error> {
        Ok(AdaptedMessage::ReadStreamEvents(dir, msg))
    }
}

impl<'a> TryFrom<(ReadDirection, raw::client_messages::ReadStreamEventsCompleted<'a>)> for AdaptedMessage<'a> {
    type Error = ((ReadDirection, raw::client_messages::ReadStreamEventsCompleted<'a>), Error);

    fn try_from((dir, msg): (ReadDirection, raw::client_messages::ReadStreamEventsCompleted<'a>)) -> Result<AdaptedMessage<'a>, Self::Error> {

        use raw::client_messages::mod_ReadStreamEventsCompleted::ReadStreamResult;

        if msg.result.is_none() {
            return Err(((dir, msg), ErrorKind::MissingResultField(ResultStatusKind::ReadStream).into()));
        }

        let next_page = if dir == ReadDirection::Backward && msg.next_event_number < 0 {
            None
        } else {
            let stream_version = match StreamVersion::try_from(msg.next_event_number) {
                                     Ok(x) => x,
                                     Err(e) => return Err( ((dir, msg), e) ),
                                 };
            Some(EventNumber::from(stream_version))
        };
        let last_event_number = match StreamVersion::try_from(msg.last_event_number) {
            Ok(x) => x,
            Err(e) => return Err( ((dir, msg), e) ),
        };

        // clone to avoid borrowing it
        let result = msg.result.as_ref().unwrap().clone();

        match result {
            ReadStreamResult::Success => {
                Ok(AdaptedMessage::ReadStreamEventsCompleted(dir, Ok(ReadStreamCompleted {
                    events: msg.events,
                    next_page: next_page,
                    last_event_number: last_event_number,
                    end_of_stream: msg.is_end_of_stream,
                    // TODO: use LogPosition
                    last_commit_position: msg.last_commit_position,
                })))
            },
            other => {
                Ok(AdaptedMessage::ReadStreamEventsCompleted(dir, Err((other, msg.error).into())))
            }
        }
    }
}

impl<'a, 'b: 'a> AsRawPayload<'a, 'b, raw::client_messages::ReadStreamEventsCompleted<'b>> for ReadStreamCompleted<'a> {
    fn as_raw(&'b self) -> raw::client_messages::ReadStreamEventsCompleted<'b> {
        use raw::client_messages::mod_ReadStreamEventsCompleted::ReadStreamResult;

        raw::client_messages::ReadStreamEventsCompleted {
            // TODO: hopefully this clone just clones the slice reference
            events: self.events.iter().map(|x| x.clone()).collect(),
            result: Some(ReadStreamResult::Success),
            next_event_number: self.next_page.map(|x| x.into()).unwrap_or(-1),
            last_event_number: self.last_event_number.into(),
            is_end_of_stream: self.end_of_stream,
            last_commit_position: self.last_commit_position,
            error: None
        }
    }
}

impl<'a, 'b: 'a> AsRawPayload<'a, 'b, raw::client_messages::ReadStreamEventsCompleted<'b>> for ReadStreamError<'a> {
    fn as_raw(&'b self) -> raw::client_messages::ReadStreamEventsCompleted<'b> {
        use raw::client_messages::mod_ReadStreamEventsCompleted::ReadStreamResult;
        use self::ReadStreamError::*;
        let (result, error) = match self {
            &NoStream => (ReadStreamResult::NoStream, None),
            &StreamDeleted => (ReadStreamResult::StreamDeleted, None),
            &NotModified => (ReadStreamResult::NotModified, None),
            // TODO: perhaps the clone could be avoided?
            &Error(Some(ref msg)) => (ReadStreamResult::Error, Some(msg.clone())),
            &Error(None) => (ReadStreamResult::Error, None),
            &AccessDenied => (ReadStreamResult::AccessDenied, None),
        };

        raw::client_messages::ReadStreamEventsCompleted {
            events: vec![],
            result: Some(result),
            next_event_number: -1,
            last_event_number: -1,
            is_end_of_stream: false,
            // TODO this could be salvaged
            last_commit_position: -1,
            error: error
        }
    }
}

impl<'a> TryFrom<(ReadDirection, raw::client_messages::ReadAllEvents)> for AdaptedMessage<'a> {
    type Error = ((ReadDirection, raw::client_messages::ReadAllEvents), Error);

    fn try_from((dir, msg): (ReadDirection, raw::client_messages::ReadAllEvents)) -> Result<AdaptedMessage<'a>, Self::Error> {
        Ok(AdaptedMessage::ReadAllEvents(dir, msg))
    }
}

impl<'a> TryFrom<(ReadDirection, raw::client_messages::ReadAllEventsCompleted<'a>)> for AdaptedMessage<'a> {
    type Error = ((ReadDirection, raw::client_messages::ReadAllEventsCompleted<'a>), Error);

    fn try_from((dir, msg): (ReadDirection, raw::client_messages::ReadAllEventsCompleted<'a>)) -> Result<AdaptedMessage<'a>, Self::Error> {
        use raw::client_messages::mod_ReadAllEventsCompleted::ReadAllResult;

        let next_commit_position = LogPosition::from_i64_opt(msg.next_commit_position);
        let next_prepare_position = LogPosition::from_i64_opt(msg.next_prepare_position);

        let res = match msg.result {
            ReadAllResult::Success => {
                Ok(ReadAllCompleted {
                    commit_position: msg.commit_position.try_into().unwrap(),
                    prepare_position: msg.prepare_position.try_into().unwrap(),
                    events: msg.events.into_iter().map(|x| {
                        let re: self::read_all::ResolvedEvent<'a> = x.into();
                        re
                    }).collect(),
                    next_commit_position: next_commit_position,
                    next_prepare_position: next_prepare_position,
                })
            },
            fail => Err((fail, msg.error).into()),
        };

        Ok(AdaptedMessage::ReadAllEventsCompleted(dir, res))
    }
}

impl<'a, 'b: 'a> AsRawPayload<'a, 'b, raw::client_messages::ReadAllEventsCompleted<'b>> for ReadAllCompleted<'a> {
    fn as_raw(&'b self) -> raw::client_messages::ReadAllEventsCompleted<'b> {
        unimplemented!()
    }
}

impl<'a, 'b: 'a> AsRawPayload<'a, 'b, raw::client_messages::ReadAllEventsCompleted<'b>> for ReadAllError<'a> {
    fn as_raw(&'b self) -> raw::client_messages::ReadAllEventsCompleted<'b> {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use raw::RawMessage;

    #[test]
    fn convert_simples() {
        let values = vec![
            (RawMessage::HeartbeatRequest, AdaptedMessage::HeartbeatRequest),
            (RawMessage::HeartbeatResponse, AdaptedMessage::HeartbeatResponse),
            (RawMessage::Ping, AdaptedMessage::Ping),
            (RawMessage::Pong, AdaptedMessage::Pong),
            (RawMessage::Authenticate, AdaptedMessage::Authenticate),
            (RawMessage::Authenticated, AdaptedMessage::Authenticated),
        ];

        for (input, expected) in values {
            test_conversions(input, expected);
        }
    }

    #[test]
    fn convert_bad_request() {
        test_conversions(
            RawMessage::BadRequest(Cow::Borrowed(&b"some_bytes"[..]).into()),
            AdaptedMessage::BadRequest(BadRequestMessage(Cow::Borrowed("some_bytes"))));

        test_conversions(
            RawMessage::BadRequest(Cow::Borrowed(&b""[..]).into()),
            AdaptedMessage::BadRequest(BadRequestMessage(Cow::Borrowed(""))));
    }

    #[test]
    fn convert_not_authenticated() {
        test_conversions(
            RawMessage::NotAuthenticated(Cow::Borrowed(&b"some_bytes"[..]).into()),
            AdaptedMessage::NotAuthenticated(Cow::Borrowed("some_bytes").into()));

        test_conversions(
            RawMessage::NotAuthenticated(Cow::Borrowed(&b""[..]).into()),
            AdaptedMessage::NotAuthenticated(Cow::Borrowed("").into()));
    }

    #[test]
    fn convert_write_events() {
        use uuid::Uuid;
        use raw::client_messages::{WriteEvents, NewEvent};

        let uuid = Uuid::new_v4();

        test_conversions(
            RawMessage::WriteEvents(WriteEvents {
                event_stream_id: Cow::Borrowed("foobar"),
                expected_version: -1,
                events: vec![NewEvent {
                    event_id: Cow::Borrowed(uuid.as_bytes()),
                    event_type: Cow::Borrowed("asdfadf"),
                    data_content_type: 0,
                    metadata_content_type: 0,
                    data: Cow::Borrowed(&b"some binary data here"[..]),
                    metadata: None,
                }],
                require_master: false,
            }),
            AdaptedMessage::WriteEvents(WriteEvents {
                event_stream_id: Cow::Borrowed("foobar"),
                expected_version: -1,
                events: vec![NewEvent {
                    event_id: Cow::Borrowed(uuid.as_bytes()),
                    event_type: Cow::Borrowed("asdfadf"),
                    data_content_type: 0,
                    metadata_content_type: 0,
                    data: Cow::Borrowed(&b"some binary data here"[..]),
                    metadata: None,
                }],
                require_master: false,
            }));
    }

    #[test]
    fn convert_write_completed() {
        use raw::client_messages::OperationResult;
        use adapted::write_events::WriteEventsCompleted;
        let body = raw::client_messages::WriteEventsCompleted {
            result: Some(OperationResult::Success),
            message: None,
            first_event_number: 0,
            last_event_number: 0,
            prepare_position: Some(100),
            commit_position: Some(100),
        };

        test_conversions(
            RawMessage::WriteEventsCompleted(body.clone()),
            AdaptedMessage::WriteEventsCompleted(Ok(WriteEventsCompleted {
                event_numbers: StreamVersion::try_from(0).unwrap()..StreamVersion::try_from(1).unwrap(),
                prepare_position: Some(LogPosition::try_from(100).unwrap()),
                commit_position: Some(LogPosition::try_from(100).unwrap()),
            })));
    }

    #[test]
    fn convert_write_failure() {
        use raw::client_messages::OperationResult;
        use adapted::write_events::WriteEventsFailure;
        use adapted::write_events::WriteEventsFailure::*;

        let errors: Vec<(OperationResult, Option<WriteEventsFailure>)> = vec![
            (OperationResult::PrepareTimeout, Some(PrepareTimeout)),
            (OperationResult::CommitTimeout, Some(CommitTimeout)),
            (OperationResult::ForwardTimeout, Some(ForwardTimeout)),
            (OperationResult::WrongExpectedVersion, Some(WrongExpectedVersion)),
            (OperationResult::StreamDeleted, Some(StreamDeleted)),
            (OperationResult::InvalidTransaction, None),
            (OperationResult::AccessDenied, Some(AccessDenied)),
        ];

        for (error, mapped) in errors {
            let body = raw::client_messages::WriteEventsCompleted {
                result: Some(error),
                message: None, // TODO: this message is lost
                first_event_number: -1,
                last_event_number: -1,
                prepare_position: None,
                commit_position: None,
            };

            match mapped {
                Some(mapped) => {
                    test_conversions(
                        RawMessage::WriteEventsCompleted(body),
                        AdaptedMessage::WriteEventsCompleted(Err(mapped)));
                },
                None => {
                    failing_conversion(RawMessage::WriteEventsCompleted(body));
                }
            }
        }
    }

    #[test]
    fn convert_read_event() {
        let body = raw::client_messages::ReadEvent {
            event_stream_id: Cow::Borrowed("asdfsd"),
            event_number: 0,
            resolve_link_tos: true,
            require_master: true
        };

        test_conversions(
            RawMessage::ReadEvent(body.clone()),
            AdaptedMessage::ReadEvent(body.clone()));
    }

    #[test]
    fn convert_bogus_read_completed() {
        use raw::client_messages::{ReadEventCompleted, ResolvedIndexedEvent, EventRecord};

        let bogus = ReadEventCompleted {
            result: None,
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
            error: None,
        };

        failing_conversion(RawMessage::ReadEventCompleted(bogus));
    }

    fn test_conversions<'a, 'b>(input: RawMessage<'a>, expected: AdaptedMessage<'b>) {
        assert_eq!(AdaptedMessage::try_from(input.clone()).unwrap(), expected);
        assert_eq!(expected.as_raw(), input);
    }

    fn failing_conversion<'a>(input: RawMessage<'a>) {
        AdaptedMessage::try_from(input).unwrap_err();
    }
}
