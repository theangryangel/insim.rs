use insim_core::{
    binrw::{self, binrw},
    string::{binrw_parse_codepage_string, binrw_write_codepage_string},
};

use crate::identifiers::RequestId;

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Insim Multiplayer - LFS sends this when a host is started or joined
pub struct Ism {
    /// Non-zero if the packet is a packet request or a reply to a request
    #[brw(pad_after = 1)]
    pub reqi: RequestId,

    /// Are we a host? false = guest, true = host
    #[brw(pad_after = 3)]
    #[br(map = |x: u8| x != 0)]
    #[bw(map = |&x| x as u8)]
    pub host: bool,

    /// Name of server joined/started
    #[br(parse_with = binrw_parse_codepage_string::<32, _>)]
    #[bw(write_with = binrw_write_codepage_string::<32, _>)]
    pub hname: String,
}
