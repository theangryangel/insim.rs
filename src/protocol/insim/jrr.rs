use super::ObjectInfo;
use deku::prelude::*;
#[cfg(feature = "serde")]
use serde::Serialize;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[deku(type = "u8", endian = "little")]
/// Used within the [Jrr] packet.
pub enum JrrAction {
    #[deku(id = "0")]
    Reject,

    #[deku(id = "1")]
    Spawn,

    #[deku(id = "2")]
    Unused2,

    #[deku(id = "3")]
    Unused3,

    #[deku(id = "4")]
    Reset,

    #[deku(id = "5")]
    ResetNoRepair,

    #[deku(id = "6")]
    Unused6,

    #[deku(id = "7")]
    Unused7,
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[deku(ctx = "_endian: deku::ctx::Endian")]
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
    pub reqi: u8,

    pub plid: u8,

    pub ucid: u8,

    #[deku(pad_bytes_after = "2")]
    pub action: JrrAction,

    pub startpos: ObjectInfo,
}
