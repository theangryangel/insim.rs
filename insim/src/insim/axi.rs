use insim_core::{
    binrw::{self, binrw},
    string::{binrw_parse_codepage_string, binrw_write_codepage_string},
};

use crate::identifiers::RequestId;

/// Auto X Info - Return information about the current layout.
// You can request information about the current layout with this IS_TINY:
// reqi: non-zero (returned in the reply)
// subtype: TINY_AXI (AutoX Info)
#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Axi {
    /// Non-zero if the packet is a packet request or a reply to a request
    #[brw(pad_after = 1)]
    pub reqi: RequestId,

    /// Autocross start position
    pub axstart: u8,

    /// Number of checkpoints
    pub numcp: u8,

    /// Number of objects
    pub numo: u16,

    /// The name of the layout last loaded (if loaded locally)
    #[br(parse_with = binrw_parse_codepage_string::<32, _>)]
    #[bw(write_with = binrw_write_codepage_string::<32, _>)]
    pub lname: String,
}
