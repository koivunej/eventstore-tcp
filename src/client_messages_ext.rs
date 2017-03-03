use std::borrow::Cow;

use client_messages::{self, WriteEvents, NewEvent};
use client_messages::mod_NotHandled::MasterInfo;

pub trait WriteEventsExt<'a> {
    /// Turns this instance to an instance that owns all it's data and thus has `'static` lifetime.
    fn into_owned(self) -> WriteEvents<'static>;
}

impl<'a> WriteEventsExt<'a> for WriteEvents<'a> {
    fn into_owned(self) -> WriteEvents<'static> {
        WriteEvents {
            event_stream_id: Cow::Owned(self.event_stream_id.into_owned()),
            expected_version: self.expected_version,
            events: self.events.into_iter().map(|x| x.into_owned()).collect(),
            require_master: self.require_master,
        }
    }
}

pub trait NewEventExt<'a> {
    /// Turns this instance to an instance that owns all it's data and thus has `'static` lifetime.
    fn into_owned(self) -> NewEvent<'static>;
}

impl<'a> NewEventExt<'a> for NewEvent<'a> {
    fn into_owned(self) -> NewEvent<'static> {
        NewEvent {
            event_id: Cow::Owned(self.event_id.into_owned()),
            event_type: Cow::Owned(self.event_type.into_owned()),
            data_content_type: self.data_content_type,
            metadata_content_type: self.metadata_content_type,
            data: Cow::Owned(self.data.into_owned()),
            metadata: self.metadata.map(|x| Cow::Owned(x.into_owned())),
        }
    }
}

pub trait MasterInfoExt<'a> {
    /// Turns this instance to an instance that owns all it's data and thus has `'static` lifetime.
    fn into_owned(self) -> client_messages::mod_NotHandled::MasterInfo<'static>;
}

impl<'a> MasterInfoExt<'a> for client_messages::mod_NotHandled::MasterInfo<'a> {
    fn into_owned(self) -> client_messages::mod_NotHandled::MasterInfo<'static> {
        MasterInfo {
            external_tcp_address: Cow::Owned(self.external_tcp_address.into_owned()),
            external_tcp_port: self.external_tcp_port,
            external_http_address: Cow::Owned(self.external_http_address.into_owned()),
            external_http_port: self.external_http_port,
            external_secure_tcp_address: self.external_secure_tcp_address.map(|x| Cow::Owned(x.into_owned())),
            external_secure_tcp_port: self.external_secure_tcp_port
        }
    }
}

pub trait ReadEventExt<'a> {
    /// Turns this instance to an instance that owns all it's data and thus has `'static` lifetime.
    fn into_owned(self) -> client_messages::ReadEvent<'static>;
}

impl<'a> ReadEventExt<'a> for client_messages::ReadEvent<'a> {
    fn into_owned(self) -> client_messages::ReadEvent<'static> {
        client_messages::ReadEvent {
            event_stream_id: Cow::Owned(self.event_stream_id.into_owned()),
            event_number: self.event_number,
            resolve_link_tos: self.resolve_link_tos,
            require_master: self.require_master,
        }
    }
}

pub trait ResolvedIndexedEventExt<'a> {
    /// Turns this instance to an instance that owns all it's data and thus has `'static` lifetime.
    fn into_owned(self) -> client_messages::ResolvedIndexedEvent<'static>;

    /// Creates an instance which borrows the non-copyable data of `self`
    fn borrowed<'b>(&'b self) -> client_messages::ResolvedIndexedEvent<'b>;

    fn as_read_event_completed<'b>(&'b self) -> client_messages::ReadEventCompleted<'b>;
}

impl<'a> ResolvedIndexedEventExt<'a> for client_messages::ResolvedIndexedEvent<'a> {
    fn into_owned(self) -> client_messages::ResolvedIndexedEvent<'static> {
        client_messages::ResolvedIndexedEvent {
            event: self.event.into_owned(),
            link: self.link.map(EventRecordExt::into_owned),
        }
    }

    fn borrowed<'b>(&'b self) -> client_messages::ResolvedIndexedEvent<'b> {
        client_messages::ResolvedIndexedEvent {
            event: self.event.borrowed(),
            link: self.link.as_ref().map(|x| x.borrowed()),
        }
    }

    fn as_read_event_completed<'b>(&'b self) -> client_messages::ReadEventCompleted<'b> {
        client_messages::ReadEventCompleted {
            result: Some(client_messages::mod_ReadEventCompleted::ReadEventResult::Success),
            event: self.borrowed(),
            error: None,
        }
    }
}

pub trait EventRecordExt<'a> {
    /// Turns this instance to an instance that owns all it's data and thus has `'static` lifetime.
    fn into_owned(self) -> client_messages::EventRecord<'static>;

    /// Creates an instance which borrows the non-copyable data of `self`
    fn borrowed<'b>(&'b self) -> client_messages::EventRecord<'b>;
}

impl<'a> EventRecordExt<'a> for client_messages::EventRecord<'a> {
    fn into_owned(self) -> client_messages::EventRecord<'static> {
        client_messages::EventRecord {
            event_stream_id: Cow::Owned(self.event_stream_id.into_owned()),
            event_number: self.event_number,
            event_id: Cow::Owned(self.event_id.into_owned()),
            event_type: Cow::Owned(self.event_type.into_owned()),
            data_content_type: self.data_content_type,
            metadata_content_type: self.metadata_content_type,
            data: Cow::Owned(self.data.into_owned()),
            metadata: self.metadata.map(|x| Cow::Owned(x.into_owned())),
            created: self.created,
            created_epoch: self.created_epoch,
        }
    }

    fn borrowed<'b>(&'b self) -> client_messages::EventRecord<'b> {
        client_messages::EventRecord {
            event_stream_id: Cow::Borrowed(&*self.event_stream_id),
            event_number: self.event_number,
            event_id: Cow::Borrowed(&*self.event_id),
            event_type: Cow::Borrowed(&*self.event_type),
            data_content_type: self.data_content_type,
            metadata_content_type: self.metadata_content_type,
            data: Cow::Borrowed(&*self.data),
            metadata: self.metadata.as_ref().map(|x| Cow::Borrowed(&**x)),
            created: self.created,
            created_epoch: self.created_epoch,
        }
    }
}

pub trait ReadStreamEventsExt<'a> {
    /// Turns this instance to an instance that owns all it's data and thus has `'static` lifetime.
    fn into_owned(self) -> client_messages::ReadStreamEvents<'static>;
}

impl<'a> ReadStreamEventsExt<'a> for client_messages::ReadStreamEvents<'a> {
    fn into_owned(self) -> client_messages::ReadStreamEvents<'static> {
        client_messages::ReadStreamEvents {
            event_stream_id: Cow::Owned(self.event_stream_id.into_owned()),
            from_event_number: self.from_event_number,
            max_count: self.max_count,
            resolve_link_tos: self.resolve_link_tos,
            require_master: self.require_master,
        }
    }
}
