//! Frame and MessageContainer

use uuid::Uuid;
use UsernamePassword;
use raw;

/// Frame in the protocol. On the wire, packets are embedded in frames with length
/// prefix and suffix.
#[derive(Debug, PartialEq, Clone)]
pub struct Package {
    /// Possible authentication data included in the packet. `Some` and `None` values of this will
    /// be used to generate corresponding `TcpFlags` first bit.
    pub authentication: Option<UsernamePassword>,
    /// Before sending an request to the server client generates a new random UUID using
    /// `uuid::Uuid::new_v4()` and later server will respond using the same `correlation_id`.
    pub correlation_id: Uuid,
    /// Enumeration of possible messages.
    pub message: raw::RawMessage<'static>,
}

trait SendReq: Send {}

impl SendReq for Package {}

