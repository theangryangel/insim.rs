use insim_core::{
    binrw::{self, binrw},
    string::{binrw_parse_codepage_string, binrw_write_codepage_string},
};

use crate::identifiers::{ConnectionId, RequestId};

bitflags::bitflags! {
    /// Additional facts about this connection. Used within [Ncn].
    #[binrw]
    #[br(map = Self::from_bits_truncate)]
    #[bw(map = |&x: &Self| x.bits())]
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Default)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize))]
    pub struct NcnFlags: u8 {
        /// User is remote
        const REMOTE = (1 << 2);
    }
}

generate_bitflag_helpers! {
    NcnFlags,
    pub is_remote => REMOTE
}

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// New Connection
pub struct Ncn {
    /// Non-zero if the packet is a packet request or a reply to a request
    pub reqi: RequestId,

    /// Unique connection id of new connection
    pub ucid: ConnectionId,

    /// LFS.net username.
    #[bw(write_with = binrw_write_codepage_string::<24, _>)]
    #[br(parse_with = binrw_parse_codepage_string::<24, _>)]
    pub uname: String,

    #[bw(write_with = binrw_write_codepage_string::<24, _>)]
    #[br(parse_with = binrw_parse_codepage_string::<24, _>)]
    /// Player Name.
    pub pname: String,

    /// true if administrative user.
    #[br(map = |x: u8| x != 0)]
    #[bw(map = |&x| x as u8)]
    pub admin: bool,

    /// Total number of connections now this player has joined, plus host
    pub total: u8,

    #[brw(pad_after = 1)]
    /// Flags describing additional facts about this connection
    pub flags: NcnFlags,
}
