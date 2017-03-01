use std::borrow::Cow;

use messages::mod_EventStore::mod_Client::mod_Messages as client_messages;
use messages::mod_EventStore::mod_Client::mod_Messages::{WriteEvents, NewEvent};
use messages::mod_EventStore::mod_Client::mod_Messages::mod_NotHandled::{NotHandledReason, MasterInfo};

use super::{Message, WriteEventsCompleted, Explanation};

pub trait WriteEventsExt<'a> {
    fn into_message(self) -> Message;
    fn into_owned(self) -> WriteEvents<'static>;
}

impl<'a> WriteEventsExt<'a> for WriteEvents<'a> {
    fn into_message(self) -> Message {
        Message::from(self)
        //Message::WriteEvents(self.into_owned())
    }

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

pub trait WriteEventsCompletedExt<'a> {
    fn into_message(self) -> Message;
}

impl<'a> WriteEventsCompletedExt<'a> for client_messages::WriteEventsCompleted<'a> {
    fn into_message(self) -> Message {
        use client_messages::OperationResult::*;
        let res = match self.result.expect("Required field was not present in WroteEventsComplete") {
            Success => {
                Ok(WriteEventsCompleted {
                    // off-by one: Range is [start, end)
                    event_numbers: self.first_event_number..self.last_event_number + 1,
                    prepare_position: self.prepare_position,
                    commit_position: self.commit_position,
                })
            }
            x => Err(Explanation::new(x, self.message.map(|x| x.into_owned()))),
        };

        Message::WriteEventsCompleted(res)
    }
}

pub trait MasterInfoExt<'a> {
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
    fn into_owned(self) -> client_messages::ResolvedIndexedEvent<'static>;

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
    fn into_owned(self) -> client_messages::EventRecord<'static>;

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
