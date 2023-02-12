use insim_core::{identifiers::RequestId, prelude::*};

#[cfg(feature = "serde")]
use serde::Serialize;

/// Enum of possible errors  that the Insim Relay can respond with.
#[derive(Debug, Clone, InsimEncode, InsimDecode)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[repr(u8)]
pub enum RelayErrorKind {
    None = 0,

    /// Packet length or structure is invalid.
    InvalidPacketLength = 1,

    /// Packet type cannot be forward to the host.
    InvalidPacketType = 2,

    /// Invalid hostname
    InvalidHostname = 3,

    /// Administrative password was rejected.
    BadAdminPassword = 4,

    /// Spectator password was rejected.
    BadSpectatorPassword = 5,

    /// Spectator password was required but not provided.
    MissingSpectatorPassword = 6,
}

impl Default for RelayErrorKind {
    fn default() -> Self {
        RelayErrorKind::None
    }
}

/// The relay will send this packet when it encounters an error.
#[derive(Debug, Clone, Default, InsimEncode, InsimDecode)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct RelayError {
    pub reqi: RequestId,

    pub err: RelayErrorKind,
}
