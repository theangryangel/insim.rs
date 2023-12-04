use insim_core::{
    binrw::{self, binrw},
    identifiers::RequestId,
};

#[cfg(feature = "serde")]
use serde::Serialize;

/// Enum of possible errors  that the Insim Relay can respond with.
#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[repr(u8)]
#[brw(repr(u8))]
pub enum RelayErrorKind {
    #[default]
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

/// The relay will send this packet when it encounters an error.
#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct RelayError {
    pub reqi: RequestId,

    pub err: RelayErrorKind,
}

impl std::fmt::Display for RelayError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Encountered a Relay Error: {:?}", self.err)
    }
}

impl std::error::Error for RelayError {}
