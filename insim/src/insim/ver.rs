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
/// Version
pub struct Version {
    #[brw(pad_after = 1)]
    pub reqi: RequestId,

    #[br(parse_with = binrw_parse_codepage_string::<8, _>)]
    #[bw(write_with = binrw_write_codepage_string::<8, _>)]
    pub version: String,

    #[br(parse_with = binrw_parse_codepage_string::<6, _>)]
    #[bw(write_with = binrw_write_codepage_string::<6, _>)]
    pub product: String,

    #[brw(pad_after = 1)]
    pub insimver: u8,
}
