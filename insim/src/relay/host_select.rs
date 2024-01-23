use insim_core::{
    binrw::{self, binrw},
    string::{binrw_parse_codepage_string, binrw_write_codepage_string},
};

use crate::identifiers::RequestId;

/// Send a HostSelect to the relay in order to start receiving information about the selected host.
#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Sel {
    /// If Non-zero LFS World relay will reply with a [crate::Packet::Ver]
    #[brw(pad_after = 1)]
    pub reqi: RequestId,

    /// Name of host to select
    #[br(parse_with = binrw_parse_codepage_string::<32, _>)]
    #[bw(write_with = binrw_write_codepage_string::<32, _>)]
    pub hname: String,

    /// Administrative password.
    #[br(parse_with = binrw_parse_codepage_string::<16, _>)]
    #[bw(write_with = binrw_write_codepage_string::<16, _>)]
    pub admin: String,

    /// Spectator password.
    #[br(parse_with = binrw_parse_codepage_string::<16, _>)]
    #[bw(write_with = binrw_write_codepage_string::<16, _>)]
    pub spec: String,
}
