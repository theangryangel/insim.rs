use insim_core::{identifiers::RequestId, binrw::{self, binrw}, string::{binrw_write_codepage_string, binrw_parse_codepage_string}};

#[cfg(feature = "serde")]
use serde::Serialize;

#[binrw]
#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[repr(u8)]
#[brw(repr(u8))]
pub enum SshError {
    #[default]
    Ok = 0,

    Dedicated = 1,

    Corrupted = 2,

    NoSave = 3,
}

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Send Screenshot
pub struct Ssh {
    pub reqi: RequestId,

    #[brw(pad_after = 4)]
    pub error: SshError,

    #[br(parse_with = binrw_parse_codepage_string::<32, _>)]
    #[bw(write_with = binrw_write_codepage_string::<32, _>)]
    pub lname: String,
}
