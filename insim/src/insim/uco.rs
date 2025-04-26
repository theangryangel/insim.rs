use std::time::Duration;

use super::{CarContact, ObjectInfo};
use crate::identifiers::{PlayerId, RequestId};

#[derive(Debug, Default, Clone, insim_macros::ReadWriteBuf)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[repr(u8)]
#[non_exhaustive]
/// Action for a [Uco] packet.
pub enum UcoAction {
    #[default]
    /// Entered a circle
    CircleEnter = 0,

    /// Left a circle
    CircleLeave = 1,

    /// Crossed checkpoint in forwards direction
    CpFwd = 2,

    /// Crossed checkpoint in backwards direction
    CpRev = 3,
}

#[derive(Debug, Clone, Default, insim_macros::ReadWriteBuf)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// User Control Object - reports crossing an InSim checkpoint / entering an InSim circle (from layout)
pub struct Uco {
    /// Non-zero if the packet is a packet request or a reply to a request
    pub reqi: RequestId,

    /// Player's unique ID that this report corresponds to.
    #[read_write_buf(pad_after = 1)]
    pub plid: PlayerId,

    /// What happened
    #[read_write_buf(pad_after = 2)]
    pub ucoaction: UcoAction,

    /// When this happened
    #[read_write_buf(duration(milliseconds = u32))]
    pub time: Duration,

    /// Was there any car contact?
    pub c: CarContact,

    /// Info about the checkpoint or circle (see below)
    pub info: ObjectInfo,
}
