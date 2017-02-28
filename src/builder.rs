use std::borrow::Cow;

use uuid::Uuid;
use package::Package;
use {UsernamePassword, Message};
use messages::mod_EventStore::mod_Client::mod_Messages::{WriteEvents, NewEvent};

pub struct Builder;

impl Builder {
    pub fn authenticate() -> AuthenticateBuilder {
        AuthenticateBuilder
    }

    pub fn write_events() -> WriteEventsBuilder {
        WriteEventsBuilder::new()
    }
}

pub struct AuthenticateBuilder;

impl AuthenticateBuilder {
    pub fn build_package(&mut self, authentication: Option<UsernamePassword>, correlation_id: Option<Uuid>) -> Package {
        Package {
            authentication: authentication,
            correlation_id: correlation_id.unwrap_or_else(|| Uuid::new_v4()),
            message: Message::Authenticate
        }
    }
}

pub struct WriteEventsBuilder {
    event_stream_id: Option<Cow<'static, str>>,
    expected_version: Option<ExpectedVersion>,
    require_master: Option<bool>,
    events: Vec<NewEvent<'static>>
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ExpectedVersion {
    /// No optimistic locking
    Any,
    /// Expect a stream not to exist
    NewStream,
    /// Expect exact number of events in the stream
    Exact(StreamVersion)
}

impl Copy for ExpectedVersion {}

impl Into<i32> for ExpectedVersion {
    fn into(self) -> i32 {
        use self::ExpectedVersion::*;
        match self {
            Any => -2,
            NewStream => -1,
            Exact(StreamVersion(x)) => x as i32
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct StreamVersion(u32);

impl Copy for StreamVersion {}

impl StreamVersion {
    pub fn from_opt(version: u32) -> Option<StreamVersion> {
        // TODO: MAX_VALUE might be some magical value, should be lower?
        if version < i32::max_value() as u32 {
            Some(StreamVersion(version))
        } else {
            None
        }
    }
}

impl WriteEventsBuilder {

    pub fn new() -> Self {
        WriteEventsBuilder {
            event_stream_id: None,
            expected_version: None,
            require_master: None,
            events: Vec::new()
        }
    }

    /// Panics if the id is an empty string
    pub fn stream_id<S: Into<Cow<'static, str>>>(&mut self, id: S) -> &mut Self {
        let id = id.into();
        assert!(id.len() > 0);
        self.event_stream_id = Some(id);
        self
    }

    pub fn expected_version(&mut self, ver: ExpectedVersion) -> &mut Self {
        self.expected_version = Some(ver);
        self
    }

    /// Should the server only handle the request if it is the cluster master. Note that while only
    /// the master server can write, other cluster members can forward request to the master.
    ///
    /// Defaults to `false`.
    pub fn require_master(&mut self, require: bool) -> &mut Self {
        self.require_master = Some(require);
        self
    }

    pub fn new_event<'b>(&'b mut self) -> NewEventBuilder<'b> {
        NewEventBuilder::new(self)
    }

    fn push_event(&mut self, event: NewEvent<'static>) -> &mut Self {
        self.events.push(event);
        self
    }

    pub fn build_command(&mut self) -> WriteEvents<'static> {
        use std::mem;

        WriteEvents {
            event_stream_id: self.event_stream_id.take().unwrap(),
            expected_version: self.expected_version.take().unwrap_or(ExpectedVersion::Any).into(),
            require_master: self.require_master.take().unwrap_or(false),
            events: mem::replace(&mut self.events, Vec::new())
        }
    }

    pub fn build_message(&mut self) -> Message {
        self.build_command().into()
    }

    pub fn build_package(&mut self, authentication: Option<UsernamePassword>, correlation_id: Option<Uuid>) -> Package {
        Package {
            authentication: authentication,
            correlation_id: correlation_id.unwrap_or_else(|| Uuid::new_v4()),
            message: self.build_command().into()
        }
    }
}

pub struct NewEventBuilder<'a> {
    parent: &'a mut WriteEventsBuilder,

    event_id: Option<Uuid>,
    event_type: Option<Cow<'static, str>>,
    data_content_type: Option<i32>,
    metadata_content_type: Option<i32>,
    data: Option<Cow<'static, [u8]>>,
    metadata: Option<Cow<'static, [u8]>>,
}

impl<'a> NewEventBuilder<'a> {
    fn new(parent: &'a mut WriteEventsBuilder) -> NewEventBuilder<'a> {
        NewEventBuilder {
            parent: parent,
            event_id: None,
            event_type: None,
            data_content_type: None,
            data: None,
            metadata_content_type: None,
            metadata: None
        }
    }

    pub fn event_id(mut self, event_id: Uuid) -> Self {
        self.event_id = Some(event_id);
        self
    }

    pub fn event_type<S: Into<Cow<'static, str>>>(mut self, ty: S) -> Self {
        // TODO: is there some invariant?
        self.event_type = Some(ty.into());
        self
    }

    pub fn data<D: Into<Vec<u8>>>(mut self, data: D) -> Self {
        self.data = Some(Cow::Owned(data.into()));
        self
    }

    pub fn data_content_type(mut self, is_json: bool) -> Self {
        self.data_content_type = if is_json { Some(1) } else { Some(0) };
        self
    }

    pub fn metadata<D: Into<Vec<u8>>>(mut self, metadata: D) -> Self {
        self.metadata = Some(Cow::Owned(metadata.into()));
        self
    }

    pub fn metadata_content_type(mut self, is_json: bool) -> Self {
        self.metadata_content_type = if is_json { Some(1) } else { Some(0) };
        self
    }

    pub fn done(self) -> &'a mut WriteEventsBuilder {
        let event = NewEvent {
            event_id: Cow::Owned(self.event_id.unwrap_or_else(|| Uuid::new_v4()).as_bytes().into_iter().cloned().collect::<Vec<_>>()),
            event_type: self.event_type.unwrap(),
            data_content_type: self.data_content_type.unwrap_or(0),
            metadata_content_type: self.metadata_content_type.unwrap_or(0),
            data: self.data.unwrap(),
            metadata: self.metadata,
        };

        self.parent.push_event(event)
    }

    pub fn cancel(self) -> &'a mut WriteEventsBuilder {
        self.parent
    }
}

#[test]
fn build_new_event_for_write_events() {
    let _ = Builder::write_events()
        .stream_id("foobar")
        .new_event()
            .event_type("foo")
            .data(vec![0u8])
            .data_content_type(false)
        .done()
        .build_package(None, None);
}
