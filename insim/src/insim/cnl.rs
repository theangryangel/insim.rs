use insim_core::binrw::{self, binrw};

use crate::identifiers::{ConnectionId, RequestId};

#[binrw]
#[derive(Debug, Default, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[repr(u8)]
#[brw(repr(u8))]
/// Used within [Cnl] to indicate the leave reason.
pub enum CnlReason {
    #[default]
    /// None
    Disconnected = 0,

    /// Timeout
    Timeout = 1,

    /// Lost Connection
    LostConnection = 2,

    /// Kicked
    Kicked = 3,

    /// Banned
    Banned = 4,

    /// Security
    Security = 5,

    /// Cheat Protection
    Cpw = 6,

    /// Out of sync with host
    Oos = 7,

    /// Join out of sync - initial sync failed
    Joos = 8,

    /// Invalid packet
    Hack = 9,
}

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
// Connection Leave
pub struct Cnl {
    /// Non-zero if the packet is a packet request or a reply to a request
    pub reqi: RequestId,

    /// Unique connection ID that left
    pub ucid: ConnectionId,

    /// Reason for disconnection
    pub reason: CnlReason,

    /// Number of remaining connections including host
    #[brw(pad_after = 2)]
    pub total: u8,
}
