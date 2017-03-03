//! Builders to help building requests.
//!
//! Every top-level builder exposes at least method `build_package` which will return a
//! ready-to-send `Package` but will panic if one or more of the required properties is not set.
//! Currently the builders are implemented so that it should support chaining.
//!
//! On `build_package` the contents are not cloned but are moved into the returned `Package`
//! which is not apparent from the use of `&mut self`.
//!
//! `UsernamePassword` can be used to authenticate specific requests. Specifying credentials for
//! every request is currently the only supported way to authenticate.
//!
//! Any `Option<Uuid>` can always be left unset, in which case `Uuid::new_v4()` (random Uuid) is
//! generated.

use std::borrow::Cow;

use uuid::Uuid;
use package::Package;
use {UsernamePassword, Message, ReadDirection};
use client_messages::{WriteEvents, NewEvent, ReadEvent, ReadStreamEvents};

/// Factory factory for creating builders.
pub struct Builder;

impl Builder {
    /// Simple builder for a Ping message, that has no other data.
    pub fn ping() -> SimpleBuilder {
        SimpleBuilder(Message::Ping)
    }

    /// Simple builder for an Authenticate message, that has no other data (credentials are passed
    /// to the `build_package` method).
    pub fn authenticate() -> SimpleBuilder {
        SimpleBuilder(Message::Authenticate)
    }

    /// Builder for `WriteEvents`, which allows writing multiple events to a stream, with expected
    /// current version of the stream.
    pub fn write_events() -> WriteEventsBuilder {
        WriteEventsBuilder::new()
    }

    /// Builder for `ReadEvent` which allows reading a single event off a stream.
    pub fn read_event() -> ReadEventBuilder {
        ReadEventBuilder::new()
    }

    /// Builder for `ReadStreamEvents` which allows reading multiple events off a stream either
    /// forwards or backwards.
    pub fn read_stream_events() -> ReadStreamEventsBuilder {
        ReadStreamEventsBuilder::new()
    }
}

/// Builder for messages without any additional contents.
pub struct SimpleBuilder(Message);

impl SimpleBuilder {
    /// Returns a package which can be sent through `EventStoreClient::call` method.
    pub fn build_package(self, authentication: Option<UsernamePassword>, correlation_id: Option<Uuid>) -> Package {
        Package {
            authentication: authentication,
            correlation_id: correlation_id.unwrap_or_else(|| Uuid::new_v4()),
            message: self.0
        }
    }
}

/// `ExpectedVersion` represents the different modes of optimistic locking when writing to a stream
/// using `WriteEventsBuilder`.
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
    /// Returns the wire representation.
    fn into(self) -> i32 {
        use self::ExpectedVersion::*;
        match self {
            Any => -2,
            NewStream => -1,
            Exact(ver) => ver.into()
        }
    }
}

impl From<StreamVersion> for ExpectedVersion {
    fn from(ver: StreamVersion) -> Self {
        ExpectedVersion::Exact(ver)
    }
}

/// `StreamVersion` represents the valid values for a stream version which is the same as the
/// event number of the latest event. As such, values are non-negative integers up to
/// `i32::max_value`. Negative values of `i32` have special meaning in the protocol, and are
/// restricted from used with this type.
///
/// Conversions to StreamVersion are quite horrible until TryFrom is stabilized.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct StreamVersion(u32);

impl Copy for StreamVersion {}

impl StreamVersion {
    /// Converts the value to a StreamVersion or panics if the value is out of range
    pub fn from(version: u32) -> Self {
        Self::from_opt(version).expect("StreamVersion overflow")
    }

    /// Converts the value to a StreamVersion returning None if the input is out of range.
    pub fn from_opt(version: u32) -> Option<Self> {
        // TODO: MAX_VALUE might be some magical value, should be lower?
        if version < i32::max_value() as u32 {
            Some(StreamVersion(version))
        } else {
            None
        }
    }

    #[doc(hidden)]
    pub fn from_i32(version: i32) -> Self {
        Self::from_i32_opt(version).expect("StreamVersion over/underflow")
    }

    #[doc(hidden)]
    pub fn from_i32_opt(version: i32) -> Option<Self> {
        if version < 0 {
            None
        } else if version == i32::max_value() {
            // this value is used to marking streams as deleted
            None
        } else {
            Some(StreamVersion(version as u32))
        }
    }
}

impl Into<i32> for StreamVersion {
    /// Returns the wire representation.
    fn into(self) -> i32 {
        self.0 as i32
    }
}

/// `EventNumber` is similar to `StreamVersion` and `ExpectedVersion` but is used when specifying a
/// position to read from in the stream. Allows specifying the first or last (when reading
/// backwards) event in addition to exact event.
#[derive(Debug, Clone, Eq)]
pub enum EventNumber {
    /// The first event in a stream
    First,
    /// Exactly the given event number
    Exact(StreamVersion),
    /// The last event in a stream
    Last,
}

impl Copy for EventNumber {}

impl PartialEq<EventNumber> for EventNumber {
    fn eq(&self, other: &EventNumber) -> bool {
        let val: i32 = (*self).into();
        let other: i32 = (*other).into();
        val == other
    }
}

impl From<StreamVersion> for EventNumber {
    fn from(ver: StreamVersion) -> Self {
        EventNumber::Exact(ver)
    }
}

impl Into<i32> for EventNumber {
    /// Returns the wire representation.
    fn into(self) -> i32 {
        match self {
            EventNumber::First => 0,
            EventNumber::Exact(x) => x.into(),
            EventNumber::Last => -1
        }
    }
}

/// Content type of the event `data` or `metadata`.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ContentType {
    /// Raw bytes
    Bytes,
    /// JSON values usable with projections in EventStore
    Json
}

impl Copy for ContentType {}

impl Into<i32> for ContentType {
    fn into(self) -> i32 {
        match self {
            ContentType::Bytes => 0,
            ContentType::Json => 1,
        }
    }
}

/// Builder for `WriteEvents` which allows writing multiple events to a stream.
///
/// # Example
///
/// ```rust
/// use eventstore_tcp::{Builder, ExpectedVersion, StreamVersion, ContentType};
/// # use eventstore_tcp::Package;
///
/// # fn example() -> Package {
/// let package = Builder::write_events()
///     .stream_id("my_stream-1")
///     .expected_version(StreamVersion::from(42))
///     .new_event()
///         .event_type("meaning_of_life")
///         .data("{ 'meaning': 42 }".as_bytes())
///         .data_content_type(ContentType::Json)
///     .done()
///     .new_event()
///         .event_type("meaning_of_life")
///         .data("{ 'meaning': 47 }".as_bytes())
///         .data_content_type(ContentType::Json)
///     .done()
///     .require_master(false) // default
///     .build_package(None, None);
/// # package
/// # }
/// ```
pub struct WriteEventsBuilder {
    event_stream_id: Option<Cow<'static, str>>,
    expected_version: Option<ExpectedVersion>,
    require_master: Option<bool>,
    events: Vec<NewEvent<'static>>
}

impl WriteEventsBuilder {

    fn new() -> Self {
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

    /// Sets the expected version of the stream as an optimistic locking mechanism.
    pub fn expected_version<V: Into<ExpectedVersion>>(&mut self, version: V) -> &mut Self {
        self.expected_version = Some(version.into());
        self
    }

    /// Should the server only handle the request if it is the cluster master. Note that while only
    /// the master server can accept writes, other cluster members can forward requests to the master.
    ///
    /// Defaults to `false`.
    pub fn require_master(&mut self, require: bool) -> &mut Self {
        self.require_master = Some(require);
        self
    }

    /// Start creating a new event using `NewEventBuilder`.
    pub fn new_event<'b>(&'b mut self) -> NewEventBuilder<'b> {
        NewEventBuilder::new(self)
    }

    fn push_event(&mut self, event: NewEvent<'static>) -> &mut Self {
        self.events.push(event);
        self
    }

    fn build_command(&mut self) -> WriteEvents<'static> {
        use std::mem;

        WriteEvents {
            event_stream_id: self.event_stream_id.take().unwrap(),
            expected_version: self.expected_version.take().unwrap_or(ExpectedVersion::Any).into(),
            require_master: self.require_master.take().unwrap_or(false),
            events: mem::replace(&mut self.events, Vec::new())
        }
    }

    fn build_message(&mut self) -> Message {
        self.build_command().into()
    }

    /// Build a package. Will panic if required values are not set.
    /// Values of this builder will be moved into the package.
    pub fn build_package(&mut self, authentication: Option<UsernamePassword>, correlation_id: Option<Uuid>) -> Package {
        build_package(self.build_message(), authentication, correlation_id)
    }
}

/// Builder for specifying an event when using `WriteEventsBuilder`.
pub struct NewEventBuilder<'a> {
    parent: &'a mut WriteEventsBuilder,

    event_id: Option<Uuid>,
    event_type: Option<Cow<'static, str>>,
    data_content_type: Option<ContentType>,
    metadata_content_type: Option<ContentType>,
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

    /// Sets the event identifier. If not specified a new random Uuid will be generated.
    pub fn event_id(mut self, event_id: Uuid) -> Self {
        self.event_id = Some(event_id);
        self
    }

    /// Specifies the event type.
    pub fn event_type<S: Into<Cow<'static, str>>>(mut self, ty: S) -> Self {
        // TODO: is there some invariant?
        self.event_type = Some(ty.into());
        self
    }

    /// Sets the data of the event.
    pub fn data<D: Into<Vec<u8>>>(mut self, data: D) -> Self {
        self.data = Some(Cow::Owned(data.into()));
        self
    }

    /// Sets the content type of the event, defaults to `ContentType::Bytes`
    pub fn data_content_type(mut self, content_type: ContentType) -> Self {
        self.data_content_type = Some(content_type);
        self
    }

    /// Sets the metadata of the event.
    pub fn metadata<D: Into<Vec<u8>>>(mut self, metadata: D) -> Self {
        self.metadata = Some(Cow::Owned(metadata.into()));
        self
    }

    /// Sets the content type of the event, defaults to `ContentType::Bytes`
    pub fn metadata_content_type(mut self, content_type: ContentType) -> Self {
        self.metadata_content_type = Some(content_type);
        self
    }

    /// Completes building a new event for `WriteEventsBuilder` by adding a new
    /// event to the builder and returning it.
    ///
    /// The server has a hard limit on the size of new events accepted, but this method currently
    /// builder currently does no size validation. Expect attempting to write over about 16MiB
    /// events (data + metadata + on-disk framing) to fail.
    pub fn done(self) -> &'a mut WriteEventsBuilder {

        fn uuid_bytes(uuid: Uuid) -> Cow<'static, [u8]> {
            Cow::Owned(uuid.as_bytes().into_iter().cloned().collect::<Vec<u8>>())
        }

        let event = NewEvent {
            event_id: uuid_bytes(self.event_id.unwrap_or_else(|| Uuid::new_v4())),
            event_type: self.event_type.unwrap(),
            data_content_type: self.data_content_type.unwrap_or(ContentType::Bytes).into(),
            metadata_content_type: self.metadata_content_type.unwrap_or(ContentType::Bytes).into(),
            data: self.data.unwrap(),
            metadata: self.metadata,
        };

        self.parent.push_event(event)
    }

    /// Cancels building this new event returning the `WriteEventsBuilder` unmodified.
    pub fn cancel(self) -> &'a mut WriteEventsBuilder {
        self.parent
    }
}

/// Builder for a single event read request `ReadEvent`.
///
/// # Example
///
/// ```rust
/// use eventstore_tcp::{Builder, StreamVersion};
/// # use eventstore_tcp::Package;
///
/// # fn example() -> Package {
/// let package = Builder::read_event()
///     .stream_id("my_stream-1")
///     .event_number(StreamVersion::from(42))
///     .resolve_link_tos(true) // default
///     .require_master(false)  // default
///     .build_package(None, None);
///
/// # package
/// # }
///
/// ```
pub struct ReadEventBuilder {
    event_stream_id: Option<Cow<'static, str>>,
    event_number: Option<EventNumber>,
    resolve_link_tos: Option<bool>,
    require_master: Option<bool>,
}

impl ReadEventBuilder {
    fn new() -> Self {
        ReadEventBuilder {
            event_stream_id: None,
            event_number: None,
            resolve_link_tos: None,
            require_master: None,
        }
    }

    /// Panics if the id is an empty string
    pub fn stream_id<S: Into<Cow<'static, str>>>(&mut self, id: S) -> &mut Self {
        let id = id.into();
        assert!(id.len() > 0);
        self.event_stream_id = Some(id);
        self
    }

    /// Event number to be read.
    pub fn event_number<N: Into<EventNumber>>(&mut self, number: N) -> &mut Self {
        self.event_number = Some(number.into());
        self
    }

    /// Whether or not the server should resolve links found in the stream to events of other
    /// streams. Defaults to `true`.
    pub fn resolve_link_tos(&mut self, resolve: bool) -> &mut Self {
        self.resolve_link_tos = Some(resolve);
        self
    }

    /// Should the server only handle the request if it is the cluster master. Master server is the
    /// only one which can accept writes, so reading from other members than master can result in
    /// some specific event not yet being replicated and thus not found.
    ///
    /// Defaults to `false`.
    pub fn require_master(&mut self, require: bool) -> &mut Self {
        self.require_master = Some(require);
        self
    }

    fn build_command(&mut self) -> ReadEvent<'static> {
        ReadEvent {
            event_stream_id: self.event_stream_id.take().unwrap(),
            event_number: self.event_number.take().unwrap().into(),
            resolve_link_tos: self.resolve_link_tos.take().unwrap_or(true),
            require_master: self.require_master.take().unwrap_or(false)
        }
    }

    fn build_message(&mut self) -> Message {
        self.build_command().into()
    }

    /// Build a package. Will panic if required values are not set.
    /// Values of this builder will be moved into the package.
    pub fn build_package(&mut self, authentication: Option<UsernamePassword>, correlation_id: Option<Uuid>) -> Package {
        build_package(self.build_message(), authentication, correlation_id)
    }
}

/// Builds a package for reading multiple events from a stream in either `ReadDirection`.
///
/// # Example
///
/// ```rust
/// use eventstore_tcp::{Builder, ReadDirection, StreamVersion};
/// # use eventstore_tcp::Package;
///
/// # fn example() -> Package {
/// let package = Builder::read_stream_events()
///     .direction(ReadDirection::Forward)
///     .stream_id("my_stream-1")
///     .from_event_number(StreamVersion::from(42))
///     .max_count(10)
///     .resolve_link_tos(true) // default
///     .require_master(false)  // default
///     .build_package(None, None);
///
/// # package
/// # }
///
/// ```
pub struct ReadStreamEventsBuilder {
    direction: Option<ReadDirection>,
    event_stream_id: Option<Cow<'static, str>>,
    from_event_number: Option<EventNumber>,
    max_count: Option<u8>,
    resolve_link_tos: Option<bool>,
    require_master: Option<bool>,
}

impl ReadStreamEventsBuilder {
    fn new() -> Self {
        ReadStreamEventsBuilder {
            direction: None,
            event_stream_id: None,
            from_event_number: None,
            max_count: None,
            resolve_link_tos: None,
            require_master: None,
        }
    }

    /// Set the read direction (required).
    pub fn direction<D: Into<ReadDirection>>(&mut self, dir: D) -> &mut Self {
        self.direction = Some(dir.into());
        self
    }

    /// Sets the maximum number of events to read (required). Panics if argument is zero.
    /// `u8` is used as larger batches should be paged. At the moment maximum buffer requirement
    /// even for 255 events is 255*16MiB > 4000MB.
    pub fn max_count(&mut self, count: u8) -> &mut Self {
        assert!(count > 0);
        // TODO: check ClientAPI, or just use u8?
        self.max_count = Some(count);
        self
    }

    /// Sets the stream to read from. Panics if the id is an empty string. Required.
    ///
    /// Panics if the id is `$all`, which is currently unimplemented.
    pub fn stream_id<S: Into<Cow<'static, str>>>(&mut self, id: S) -> &mut Self {
        let id = id.into();
        if id == "$all" {
            // ReadAll is not yet implemented
            unimplemented!()
        }
        assert!(id.len() > 0);
        self.event_stream_id = Some(id);
        self
    }

    /// Event number to read from to the given direction.
    pub fn from_event_number<N: Into<EventNumber>>(&mut self, n: N) -> &mut Self {
        self.from_event_number = Some(n.into());
        self
    }

    /// Whether or not the server should resolve links found in the stream to events of other
    /// streams. Defaults to `true`.
    pub fn resolve_link_tos(&mut self, resolve: bool) -> &mut Self {
        self.resolve_link_tos = Some(resolve);
        self
    }

    /// Should the server only handle the request if it is the cluster master. Master server is the
    /// only one which can accept writes, so reading from other members than master can result in
    /// some specific event not yet being replicated and thus not found.
    ///
    /// Defaults to `false`.
    pub fn require_master(&mut self, require: bool) -> &mut Self {
        self.require_master = Some(require);
        self
    }

    fn build_command(&mut self) -> ReadStreamEvents<'static> {
        ReadStreamEvents {
            event_stream_id: self.event_stream_id.take().expect("event_stream_id not set"),
            from_event_number: self.from_event_number.take().expect("from_event_number not set").into(),
            max_count: self.max_count.unwrap_or(10) as i32,
            resolve_link_tos: self.resolve_link_tos.unwrap_or(true),
            require_master: self.require_master.unwrap_or(false)
        }
    }

    fn build_message(&mut self) -> Message {
        Message::ReadStreamEvents(self.direction.expect("direction not set"), self.build_command())
    }

    /// Build a package. Will panic if required values are not set.
    /// Values of this builder will be moved into the package.
    ///
    /// Note that this method can start panicing in future if an unsupported combination of `ReadDirection`
    /// and `from_event_number` is specified.
    pub fn build_package(&mut self, authentication: Option<UsernamePassword>, correlation_id: Option<Uuid>) -> Package {
        build_package(self.build_message(), authentication, correlation_id)
    }
}

fn build_package<M: Into<Message>>(msg: M, authentication: Option<UsernamePassword>, correlation_id: Option<Uuid>) -> Package {
    Package {
        authentication: authentication,
        correlation_id: correlation_id.unwrap_or_else(|| Uuid::new_v4()),
        message: msg.into()
    }
}

#[test]
fn build_new_event_for_write_events() {
    let _ = Builder::write_events()
        .stream_id("foobar")
        .new_event()
            .event_type("foo")
            .data(vec![0u8])
            .data_content_type(ContentType::Bytes)
        .done()
        .build_package(None, None);
}
