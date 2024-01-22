use crate::identifiers::{ConnectionId, RequestId};
use insim_core::binrw::{self, binrw};

#[binrw]
#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[repr(u8)]
#[brw(repr(u8))]
/// [Ttc] subtype.
pub enum TtcType {
    /// Send Axm for the current layout editor selection
    #[default]
    Sel = 1,

    /// Send Axm every time the selection changes
    SelStart = 2,

    /// Stop sending Axm's
    SelStop = 3,
}

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// General purpose Target To Connection packet
/// b1..b3 may be used in various ways, depending on the subtype
pub struct Ttc {
    /// Non-zero if the packet is a packet request or a reply to a request
    pub reqi: RequestId,
    /// Subtype
    pub subt: TtcType,

    /// Connection unique ID to target
    pub ucid: ConnectionId,

    // TODO: Fix this. It should be rolled into TtcType
    /// B1, B2, B3 may be used in various ways depending on SubT
    pub b1: u8,
    /// B1, B2, B3 may be used in various ways depending on SubT
    pub b2: u8,
    /// B1, B2, B3 may be used in various ways depending on SubT
    pub b3: u8,
}
