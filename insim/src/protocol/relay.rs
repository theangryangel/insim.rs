//! Definitions for the Insim Relay packets.
//! The InSim Relay is a service that can connect to your LFS host via Insim and relay the InSim
//! information sent by your host, to anyone who connects to the Insim Relay.
//!
//! This relayed data can be used by programmers for various things, such as the LFS Remote
//! (remote viewing / adminning of a race) and race-tracking to store race information and
//! statistics.
//!
//! See [https://en.lfsmanual.net/wiki/InSim_Relay](https://en.lfsmanual.net/wiki/InSim_Relay) for more information.

#[cfg(feature = "serde")]
use serde::Serialize;

use bitflags::bitflags;

use crate::string::{istring, CodepageString};
use crate::track::Track;

use super::identifiers::RequestId;

/// Ask the relay if we are logged in as an administrative user on the selected host. A
/// [AdminResponse] is sent back by the relay.
#[derive(Debug, Clone, Default, InsimEncode, InsimDecode)]
pub struct AdminRequest {
    #[insim(pad_bytes_after = "1")]
    pub reqi: RequestId,
}

/// Reponse to a [AdminRequest] packet, indicating if we are logged in as an administrative user on
/// the selected host.
#[derive(Debug, Clone, Default, InsimEncode, InsimDecode)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct AdminResponse {
    /// Optional request identifier. If a request identifier was sent in the request, it will be
    /// included in any relevant response packet.
    pub reqi: RequestId,

    pub admin: u8,
}

/// Request a list of available hosts from the Insim Relay. After sending this packet the relay
/// will respond with a HostList packet.
#[derive(Debug, Clone, Default, InsimEncode, InsimDecode)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct HostListRequest {
    #[insim(pad_bytes_after = "1")]
    pub reqi: RequestId,
}

bitflags! {
    /// Bitwise flags used within the [HostInfo] packet, which is in turn used by the [HostList]
    /// packet.
    #[cfg_attr(feature = "serde", derive(Serialize))]
    pub struct HostInfoFlags: u8 {
        SPECTATE_PASSWORD_REQUIRED => (1 << 0),
        LICENSED => (1 << 1),
        S1 => (1 << 2),
        S2 => (1 << 3),
        FIRST => (1 << 6),
        LAST => (1 << 7),
    }
}

/// Information about a host. Used within the [HostList] packet.
#[derive(Debug, Clone, Default, InsimEncode, InsimDecode)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct HostInfo {
    #[insim(bytes = "32")]
    pub hname: CodepageString,

    pub track: Track,

    pub flags: HostInfoFlags,

    pub numconns: u8,
}

/// The relay will send a list of available hosts using this packet. There may be more than one
/// HostList packet sent in response to a [HostListRequest]. You may use the [HostInfoFlags] to
/// determine if the host is the last in the list.
#[derive(Debug, Clone, Default, InsimEncode, InsimDecode)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct HostList {
    pub reqi: RequestId,

    pub numhosts: u8,

    #[insim(count = "numhosts")]
    pub hinfo: Vec<HostInfo>,
}

/// Send a HostSelect to the relay in order to start receiving information about the selected host.
#[derive(Debug, Clone, Default, InsimEncode, InsimDecode)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct HostSelect {
    #[insim(pad_bytes_after = "1")]
    pub reqi: RequestId,

    #[insim(bytes = "32")]
    pub hname: CodepageString,

    pub admin: String,

    pub spec: String,
}

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
