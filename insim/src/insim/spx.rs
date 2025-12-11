use std::time::Duration;

use super::{Fuel200, PenaltyInfo};
use crate::identifiers::{PlayerId, RequestId};

#[derive(Debug, Clone, Default, insim_core::Decode, insim_core::Encode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Split X timing
pub struct Spx {
    /// Non-zero if the packet is a packet request or a reply to a request
    pub reqi: RequestId,

    /// Unique player id for this the split timing
    pub plid: PlayerId,

    #[insim(duration = u32)]
    /// Split duration
    pub stime: Duration,

    #[insim(duration = u32)]
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_spx() {
        assert_from_to_bytes!(
            Spx,
            [
                0,  // reqi
                2,  // plid
                4,  // stime (1)
                0,  // stime (2)
                0,  // stime (3)
                1,  // stime (4)
                64, // etime (1)
                0,  // etime (2)
                1,  // etime (3)
                0,  // etime (4)
                3,  // split
                6,  // penalty
                3,  // numstops
                40, // fuel200
            ],
            |spx: Spx| {
                assert_eq!(spx.reqi, RequestId(0));
                assert_eq!(spx.plid, PlayerId(2));
                assert_eq!(spx.stime, Duration::from_millis(16777220));
                assert_eq!(spx.etime, Duration::from_millis(65600));
                assert_eq!(spx.split, 3);
                assert_eq!(spx.penalty, PenaltyInfo::Seconds45);
                assert_eq!(spx.numstops, 3);
                assert!(matches!(spx.fuel200, Fuel200::Percentage(40)));
            }
        )
    }
}
