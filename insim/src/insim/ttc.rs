use insim_core::{
    binrw::{self, binrw},
    identifiers::{ConnectionId, RequestId},
};

#[cfg(feature = "serde")]
use serde::Serialize;

#[binrw]
#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[repr(u8)]
#[brw(repr(u8))]
pub enum TtcType {
    #[default]
    None = 0,

    /// Send Axm for the current layout editor selection
    Sel = 1,

    /// Send Axm every time the selection changes
    SelStart = 2,

    /// Stop sending Axm's
    SelStop = 3,
}

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// General purpose Target To Connection packet
/// b1..b3 may be used in various ways, depending on the subtype
pub struct Ttc {
    pub reqi: RequestId,
    pub subt: TtcType,

    pub ucid: ConnectionId,
    pub b1: u8,
    pub b2: u8,
    pub b3: u8,
}
