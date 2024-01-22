use insim_core::binrw::{self, binrw};

use crate::identifiers::{ConnectionId, PlayerId, RequestId};

use super::ObjectInfo;

#[binrw]
#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[repr(u8)]
#[brw(repr(u8))]
/// Used within the [Jrr] packet.
pub enum JrrAction {
    #[default]
    /// Reject the join request
    Reject = 0,

    /// Allow the user to spawn
    Spawn = 1,

    /// Move the player
    Reset = 4,

    /// Move the player, but do not repair
    ResetNoRepair = 5,
}

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Join Request Reply
/// Set the ISF_REQ_JOIN flag in the IS_ISI to receive join requests
///
/// A join request is seen as an IS_NPL packet with ZERO in the NumP field
/// An immediate response (e.g. within 1 second) is required using an IS_JRR packet
/// In this case, PLID must be zero and JRRAction must be JRR_REJECT or JRR_SPAWN
/// If you allow the join and it is successful you will then get a normal IS_NPL with NumP set
/// You can also specify the start position of the car using the StartPos structure
///
/// IS_JRR can also be used to move an existing car to a different location
/// In this case, PLID must be set, JRRAction must be JRR_RESET or higher and StartPos must be set
pub struct Jrr {
    /// Non-zero if the packet is a packet request or a reply to a request
    pub reqi: RequestId,

    /// Player's unique ID
    pub plid: PlayerId,

    /// Unique connection ID
    pub ucid: ConnectionId,

    #[brw(pad_after = 2)]
    /// Action taken/to take
    pub action: JrrAction,

    /// 0: use default start point / Flags = 0x80: set start point
    pub startpos: ObjectInfo,
}
