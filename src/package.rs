use uuid::Uuid;
use UsernamePassword;
use errors::Error;
use raw;
use adapted;

bitflags!{
    pub flags TcpFlags: u8 {
        const FLAG_NONE = 0x00,
        const FLAG_AUTHENTICATED = 0x01,
        //const FLAG_TRUSTED_WRITE = 0x02, // only in core
    }
}

/// Frame in the protocol. On the wire, packets are embedded in frames with length
/// prefix and suffix.
#[derive(Debug, PartialEq)]
pub struct Package {
    /// Possible authentication data included in the packet. `Some` and `None` values of this will
    /// be used to generate corresponding `TcpFlags` first bit.
    pub authentication: Option<UsernamePassword>,
    /// Before sending an request to the server client generates a new random UUID using
    /// `uuid::Uuid::new_v4()` and later server will respond using the same `correlation_id`.
    pub correlation_id: Uuid,
    /// Enumeration of possible messages.
    pub message: MessageContainer<'static>,
}

/// Wrapper for either a raw just decoded or built-for sending message or an adapted one.
#[derive(Debug, PartialEq)]
pub enum MessageContainer<'a> {
    Raw(raw::RawMessage<'a>),
    Adapted(adapted::AdaptedMessage<'a>),
}

impl<'a> From<raw::RawMessage<'a>> for MessageContainer<'a> {
    fn from(raw: raw::RawMessage<'a>) -> MessageContainer<'a> {
        MessageContainer::Raw(raw)
    }
}

impl<'a> From<adapted::AdaptedMessage<'a>> for MessageContainer<'a> {
    fn from(adapted: adapted::AdaptedMessage<'a>) -> MessageContainer<'a> {
        MessageContainer::Adapted(adapted)
    }
}

impl<'a> MessageContainer<'a> {
    /// Attempt to convert a raw message into an adapted one
    pub fn try_adapt(self) -> Result<adapted::AdaptedMessage<'a>, (raw::RawMessage<'a>, Error)> {
        use CustomTryInto;
        use self::MessageContainer::*;
        match self {
            Adapted(adapted) => Ok(adapted),
            Raw(raw) => {
                raw.try_into()
            }
        }
    }
}
