use uuid::Uuid;
use {Message, UsernamePassword};

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
    pub message: Message,
}
