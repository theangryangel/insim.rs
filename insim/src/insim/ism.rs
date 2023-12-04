use insim_core::{
    binrw::{self, binrw},
    identifiers::RequestId,
    string::{binrw_parse_codepage_string, binrw_write_codepage_string},
};

#[cfg(feature = "serde")]
use serde::Serialize;

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Insim Multiplayer - LFS sends this when a host is started or joined
pub struct Ism {
    #[brw(pad_after = 1)]
    pub reqi: RequestId,

    #[brw(pad_after = 3)]
    /// false = guest, true = host
    // FIXME should be bool
    pub host: u8,

    #[br(parse_with = binrw_parse_codepage_string::<32, _>)]
    #[bw(write_with = binrw_write_codepage_string::<32, _>)]
    pub hname: String,
}
