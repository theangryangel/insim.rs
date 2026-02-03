use std::time::Duration;

use super::CarContact;
use crate::identifiers::{PlayerId, RequestId};

#[derive(Debug, Default, Clone, insim_core::Decode, insim_core::Encode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[repr(u8)]
#[non_exhaustive]
/// Action type reported by [Csc].
pub enum CscAction {
    #[default]
    /// Stopped
    Stop = 0,

    /// Started
    Start = 1,
}

#[derive(Debug, Clone, Default, insim_core::Decode, insim_core::Encode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Car state changed event.
///
/// - Reports start/stop transitions for a car.
pub struct Csc {
    /// Request identifier echoed by replies.
    pub reqi: RequestId,

    /// Player that changed state.
    #[insim(pad_after = 1)]
    pub plid: PlayerId,

    /// State change action.
    #[insim(pad_after = 2)]
    pub cscaction: CscAction,

    /// Time since session start (wraps periodically).
    #[insim(duration = u32)]
    pub time: Duration,

    /// Contact details, if relevant.
    pub c: CarContact,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_csc() {
        assert_from_to_bytes!(
            Csc,
            [
                0,   // reqi
                2,   // plid
                0,   // sp0
                1,   // cscaction
                0,   // sp2
                0,   // sp3
                178, // time (1)
                122, // time (2)
                0,   // time (3)
                0,   // time (4)
                4,   // c - direction
                3,   // c - heading
                1,   // c - speed
                39,  // c - zbyte
                104, // c - x (1)
                246, // c - x (2)
                207, // c - y (1)
                5,   // c - y (2)
            ],
            |csc: Csc| {
                assert_eq!(csc.reqi, RequestId(0));
                assert_eq!(csc.time, Duration::from_millis(31410));
            }
        );
    }
}
