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
        Message::WriteEvents(self.into_owned())
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

