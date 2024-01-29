use insim_core::{
    binrw::{self, binrw},
    string::{binrw_parse_codepage_string, binrw_write_codepage_string},
};

use crate::identifiers::RequestId;

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Version packet - informational
/// It is advisable to request version information as soon as you have connected, to
/// avoid problems when connecting to a host with a later or earlier version.  You will
/// be sent a version packet on connection if you set ReqI in the IS_ISI packet.
pub struct Ver {
    /// Non-zero if the packet is a packet request or a reply to a request
    #[brw(pad_after = 1)]
    pub reqi: RequestId,

    /// LFS version, e.g. 0.3G
    #[br(parse_with = binrw_parse_codepage_string::<8, _>)]
    #[bw(write_with = binrw_write_codepage_string::<8, _>)]
    pub version: String,

    /// Product: DEMO / S1 / S2 / S3
    #[br(parse_with = binrw_parse_codepage_string::<6, _>)]
    #[bw(write_with = binrw_write_codepage_string::<6, _>)]
    pub product: String,

    /// InSim version
    #[brw(pad_after = 1)]
    pub insimver: u8,
}
