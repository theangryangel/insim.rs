use insim_core::{
    identifiers::{ConnectionId, PlayerId, RequestId},
    prelude::*,
};

#[cfg(feature = "serde")]
use serde::Serialize;

use super::ObjectInfo;

#[derive(Debug, InsimEncode, InsimDecode, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[repr(u8)]
/// Used within the [Jrr] packet.
pub enum JrrAction {
    Reject = 0,

    Spawn = 1,

    Unused2 = 2,

    Unused3 = 3,

    Reset = 4,

    ResetNoRepair = 5,

    Unused6 = 6,

    Unused7 = 7,
}

impl Default for JrrAction {
    fn default() -> Self {
        JrrAction::Reject
    }
}

#[derive(Debug, InsimEncode, InsimDecode, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
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
    pub reqi: RequestId,

    pub plid: PlayerId,

    pub ucid: ConnectionId,

    #[insim(pad_bytes_after = "2")]
    pub action: JrrAction,

    pub startpos: ObjectInfo,
}
