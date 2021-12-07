//! Definitions for the Insim Relay packets.
//! The InSim Relay is a service that can connect to your LFS host via Insim and relay the InSim
//! information sent by your host, to anyone who connects to the Insim Relay.
//!
//! This relayed data can be used by programmers for various things, such as the LFS Remote
//! (remote viewing / adminning of a race) and race-tracking to store race information and
//! statistics.
//!
//! See [https://en.lfsmanual.net/wiki/InSim_Relay](https://en.lfsmanual.net/wiki/InSim_Relay) for more information.

use crate::packet_flags;
use crate::string::{ICodepageString, IString};
use crate::track::Track;
use deku::prelude::*;
use serde::Serialize;

/// Ask the relay if we are logged in as an administrative user on the selected host. A
/// [AdminResponse] is sent back by the relay.
#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Serialize)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
pub struct AdminRequest {
    #[deku(pad_bytes_after = "1")]
    pub reqi: u8,
}

/// Reponse to a [AdminRequest] packet, indicating if we are logged in as an administrative user on
/// the selected host.
#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Serialize)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
pub struct AdminResponse {
    /// Optional request identifier. If a request identifier was sent in the request, it will be
    /// included in any relevant response packet.
    pub reqi: u8,

    pub admin: u8,
}

/// Request a list of available hosts from the Insim Relay. After sending this packet the relay
/// will respond with a HostList packet.
#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Default, Serialize)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
pub struct HostListRequest {
    #[deku(pad_bytes_after = "1")]
    pub reqi: u8,
}

packet_flags! {
    /// Bitwise flags used within the [HostInfo] packet, which is in turn used by the [HostList]
    /// packet.
    #[derive(Serialize)]
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
#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Serialize)]
pub struct HostInfo {
    #[deku(bytes = "32")]
    pub hname: ICodepageString,

    pub track: Track,

    #[deku(bytes = "1")]
    pub flags: HostInfoFlags,

    #[deku(bytes = "1")]
    pub numconns: u8,
}

/// The relay will send a list of available hosts using this packet. There may be more than one
/// HostList packet sent in response to a [HostListRequest]. You may use the [HostInfoFlags] to
/// determine if the host is the last in the list.
#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Serialize)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
pub struct HostList {
    pub reqi: u8,

    pub numhosts: u8,

    #[deku(count = "numhosts")]
    pub hinfo: Vec<HostInfo>,
}

/// Send a HostSelect to the relay in order to start receiving information about the selected host.
#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Default, Serialize)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
pub struct HostSelect {
    #[deku(pad_bytes_after = "1")]
    pub reqi: u8,

    #[deku(bytes = "32")]
    pub hname: ICodepageString,

    #[deku(bytes = "16")]
    pub admin: IString,

    #[deku(bytes = "16")]
    pub spec: IString,
}

/// Enum of possible errors  that the Insim Relay can respond with.
#[derive(Debug, PartialEq, DekuRead, DekuWrite, Serialize, Clone)]
#[deku(type = "u8", endian = "little")]
pub enum ErrorType {
    #[deku(id = "0")]
    None,

    /// Packet length or structure is invalid.
    #[deku(id = "1")]
    InvalidPacketLength,

    /// Packet type cannot be forward to the host.
    #[deku(id = "2")]
    InvalidPacketType,

    /// Invalid hostname
    #[deku(id = "3")]
    InvalidHostname,

    /// Administrative password was rejected.
    #[deku(id = "4")]
    BadAdminPassword,

    /// Spectator password was rejected.
    #[deku(id = "5")]
    BadSpectatorPassword,

    /// Spectator password was required but not provided.
    #[deku(id = "6")]
    MissingSpectatorPassword,
}

/// The relay will send this packet when it encounters an error.
#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Serialize)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
pub struct Error {
    pub reqi: u8,

    pub err: ErrorType,
}
