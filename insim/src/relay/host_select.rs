use insim_core::{identifiers::RequestId, binrw::{self, binrw}, string::{binrw_parse_codepage_string, binrw_write_codepage_string}};

#[cfg(feature = "serde")]
use serde::Serialize;

/// Send a HostSelect to the relay in order to start receiving information about the selected host.
#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct HostSelect {
    #[brw(pad_after = 1)]
    pub reqi: RequestId,

    #[br(parse_with = binrw_parse_codepage_string::<32, _>)]
    #[bw(write_with = binrw_write_codepage_string::<32, _>)]
    pub hname: String,

    #[br(parse_with = binrw_parse_codepage_string::<16, _>)]
    #[bw(write_with = binrw_write_codepage_string::<16, _>)]
    pub admin: String,

    #[br(parse_with = binrw_parse_codepage_string::<16, _>)]
    #[bw(write_with = binrw_write_codepage_string::<16, _>)]
    pub spec: String,
}
