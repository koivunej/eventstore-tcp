use std::borrow::Cow;

use raw::client_messages::{self, WriteEvents, NewEvent};
use raw::client_messages::mod_NotHandled::MasterInfo;

pub trait WriteEventsExt<'a> {
    /// Turns this instance to an instance that owns all it's data and thus has `'static` lifetime.
    fn into_owned(self) -> WriteEvents<'static>;
}

impl<'a> WriteEventsExt<'a> for client_messages::WriteEvents<'a> {
    fn into_owned(self) -> client_messages::WriteEvents<'static> {
        client_messages::WriteEvents {
            event_stream_id: Cow::Owned(self.event_stream_id.into_owned()),
            expected_version: self.expected_version,
            events: self.events.into_iter().map(|x| x.into_owned()).collect(),
            require_master: self.require_master,
        }
    }
}

pub trait WriteEventsCompletedExt<'a> {
    /// Turns this instance to an instance that owns all it's data and thus has `'static` lifetime.
    fn into_owned(self) -> client_messages::WriteEventsCompleted<'static>;
}

impl<'a> WriteEventsCompletedExt<'a> for client_messages::WriteEventsCompleted<'a> {
    fn into_owned(self) -> client_messages::WriteEventsCompleted<'static> {
        client_messages::WriteEventsCompleted {
            result: self.result,
            message: owned_opt_cow(self.message),
            first_event_number: self.first_event_number,
            last_event_number: self.last_event_number,
            prepare_position: self.prepare_position,
            commit_position: self.commit_position,
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

pub trait ReadEventCompletedExt<'a> {
    /// Turns this instance to an instance that owns all it's data and thus has `'static` lifetime.
    fn into_owned(self) -> client_messages::ReadEventCompleted<'static>;
}

impl<'a> ReadEventCompletedExt<'a> for client_messages::ReadEventCompleted<'a> {
    fn into_owned(self) -> client_messages::ReadEventCompleted<'static> {
        client_messages::ReadEventCompleted {
            result: self.result,
            event: self.event.into_owned(),
            error: owned_opt_cow(self.error),
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

pub trait ResolvedEventExt<'a> {
    fn into_owned(self) -> client_messages::ResolvedEvent<'static>;

    fn borrowed<'b>(&'b self) -> client_messages::ResolvedEvent<'b>;
}

impl<'a> ResolvedEventExt<'a> for client_messages::ResolvedEvent<'a> {
    fn into_owned(self) -> client_messages::ResolvedEvent<'static> {
        client_messages::ResolvedEvent {
            event: self.event.into_owned(),
            link: self.link.map(|x| x.into_owned()),
            commit_position: self.commit_position,
            prepare_position: self.prepare_position,
        }
    }

    fn borrowed<'b>(&'b self) -> client_messages::ResolvedEvent<'b> {
        client_messages::ResolvedEvent {
            event: self.event.borrowed(),
            link: self.link.as_ref().map(|x| x.borrowed()),
            commit_position: self.commit_position,
            prepare_position: self.prepare_position,
        }
    }
}

pub trait NotHandledExt<'a> {
    fn into_owned(self) -> client_messages::NotHandled<'static>;
    fn borrowed<'b>(&'b self) -> client_messages::NotHandled<'b>;
}

impl<'a> NotHandledExt<'a> for client_messages::NotHandled<'a> {
    fn into_owned(self) -> client_messages::NotHandled<'static> {
        client_messages::NotHandled {
            reason: self.reason,
            additional_info: owned_opt_cow(self.additional_info),
        }
    }

    fn borrowed<'b>(&'b self) -> client_messages::NotHandled<'b> {
        client_messages::NotHandled {
            reason: self.reason,
            additional_info: borrowed_opt_cow(&self.additional_info),
        }
    }
}

pub trait ReadStreamEventsCompletedExt<'a> {
    fn into_owned(self) -> client_messages::ReadStreamEventsCompleted<'static>;
    fn borrowed<'b>(&'b self) -> client_messages::ReadStreamEventsCompleted<'b>;
}

impl<'a> ReadStreamEventsCompletedExt<'a> for client_messages::ReadStreamEventsCompleted<'a> {
    fn into_owned(self) -> client_messages::ReadStreamEventsCompleted<'static> {
        client_messages::ReadStreamEventsCompleted {
            events: self.events.into_iter().map(|x| x.into_owned()).collect(),
            result: self.result,
            next_event_number: self.next_event_number,
            last_event_number: self.last_event_number,
            is_end_of_stream: self.is_end_of_stream,
            last_commit_position: self.last_commit_position,
            error: owned_opt_cow(self.error),
        }
    }

    fn borrowed<'b>(&'b self) -> client_messages::ReadStreamEventsCompleted<'b> {
        client_messages::ReadStreamEventsCompleted {
            events: self.events.iter().map(|x| x.borrowed()).collect(),
            result: self.result,
            next_event_number: self.next_event_number,
            last_event_number: self.last_event_number,
            is_end_of_stream: self.is_end_of_stream,
            last_commit_position: self.last_commit_position,
            error: borrowed_opt_cow(&self.error),
        }
    }
}

pub trait ReadAllEventsCompletedExt<'a> {
    fn into_owned(self) -> client_messages::ReadAllEventsCompleted<'static>;
    fn borrowed<'b>(&'b self) -> client_messages::ReadAllEventsCompleted<'b>;
}

impl<'a> ReadAllEventsCompletedExt<'a> for client_messages::ReadAllEventsCompleted<'a> {
    fn into_owned(self) -> client_messages::ReadAllEventsCompleted<'static> {
        client_messages::ReadAllEventsCompleted {
            commit_position: self.commit_position,
            prepare_position: self.prepare_position,
            events: self.events.into_iter().map(|x| x.into_owned()).collect(),
            next_commit_position: self.next_commit_position,
            next_prepare_position: self.next_prepare_position,
            result: self.result,
            error: owned_opt_cow(self.error),
        }
    }

    fn borrowed<'b>(&'b self) -> client_messages::ReadAllEventsCompleted<'b> {
        client_messages::ReadAllEventsCompleted {
            commit_position: self.commit_position,
            prepare_position: self.prepare_position,
            events: self.events.iter().map(|x| x.borrowed()).collect(),
            next_commit_position: self.next_commit_position,
            next_prepare_position: self.next_prepare_position,
            result: self.result,
            error: borrowed_opt_cow(&self.error),
        }
    }
}

fn owned_opt_cow<'a, T: ToOwned + ?Sized>(opt_cow: Option<Cow<'a, T>>) -> Option<Cow<'static, T>> {
    opt_cow.map(|x| Cow::Owned(x.into_owned()))
}

fn borrowed_opt_cow<'a, 'b, T: ToOwned + ?Sized>(opt_cow: &'b Option<Cow<'a, T>>) -> Option<Cow<'b, T>> {
    opt_cow.as_ref().map(|x| Cow::Borrowed(x.as_ref()))
}
