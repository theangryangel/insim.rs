use insim_core::{
    binrw::{self, binrw},
    identifiers::RequestId,
    string::{binrw_parse_codepage_string, binrw_write_codepage_string},
};

#[cfg(feature = "serde")]
use serde::Serialize;

/// Auto X Info
#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct Axi {
    #[brw(pad_after = 1)]
    pub reqi: RequestId,

    pub axstart: u8,
    pub numcp: u8,
    pub numo: u16,

    #[br(parse_with = binrw_parse_codepage_string::<32, _>)]
    #[bw(write_with = binrw_write_codepage_string::<32, _>)]
    pub lname: String,
}
