use std::time::Duration;

use super::{Fuel200, PenaltyInfo};
use crate::identifiers::{PlayerId, RequestId};

#[derive(Debug, Clone, Default, insim_macros::ReadWriteBuf)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Split X timing
pub struct Spx {
    /// Non-zero if the packet is a packet request or a reply to a request
    pub reqi: RequestId,

    /// Unique player id for this the split timing
    pub plid: PlayerId,

    #[read_write_buf(duration(milliseconds = u32))]
    /// Split duration
    pub stime: Duration,

    #[read_write_buf(duration(milliseconds = u32))]
    /// Total elapsed time
    pub etime: Duration,

    /// Split number
    pub split: u8,

    /// Any penalties the user has received
    pub penalty: PenaltyInfo,

    /// The number of stops taken
    pub numstops: u8,

    /// When /showfuel yes: double fuel percent / no: 255
    pub fuel200: Fuel200,
}
