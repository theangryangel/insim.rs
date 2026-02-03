use std::time::Duration;

use super::{Fuel200, PenaltyInfo};
use crate::identifiers::{PlayerId, RequestId};

#[derive(Debug, Clone, Default, insim_core::Decode, insim_core::Encode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Split timing for a player.
///
/// - Sent when a player crosses a split.
pub struct Spx {
    /// Request identifier echoed by replies.
    pub reqi: RequestId,

    /// Player for this split.
    pub plid: PlayerId,

    #[insim(duration = u32)]
    /// Split time.
    pub stime: Duration,

    #[insim(duration = u32)]
    /// Total elapsed time since session start.
    pub etime: Duration,

    /// Split index (1-3).
    pub split: u8,

    /// Current penalty state.
    pub penalty: PenaltyInfo,

    /// Number of pit stops.
    pub numstops: u8,

    /// Fuel remaining (double-percentage when enabled).
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
