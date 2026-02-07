use std::time::Duration;

use super::CarContact;
use crate::identifiers::{PlayerId, RequestId};

#[derive(Debug, Default, Clone, insim_core::Decode, insim_core::Encode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
#[non_exhaustive]
/// Hot lap validity failure reason.
pub enum Hlvc {
    /// Ground
    #[default]
    Ground = 0,

    /// Wall
    Wall = 1,

    /// Speeding in pitlane
    Speeding = 4,

    /// Out of bounds
    OutOfBounds = 5,
}

#[derive(Debug, Clone, Default, insim_core::Decode, insim_core::Encode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Hot lap validity violation report.
///
/// - Sent when HLVC is enabled in [IsiFlags](crate::insim::IsiFlags).
pub struct Hlv {
    /// Request identifier echoed by replies.
    pub reqi: RequestId,

    /// Player involved in the incident.
    pub plid: PlayerId,

    /// Reason the lap was invalidated.
    #[insim(pad_after = 3)]
    pub hlvc: Hlvc,

    #[insim(duration = u32)]
    /// Time since session start (wraps periodically).
    pub time: Duration,

    /// Contact details, if relevant.
    pub c: CarContact,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_hlv() {
        assert_from_to_bytes!(
            Hlv,
            [
                0,   // reqi
                3,   // plid
                1,   // hlvc
                0,   // sp1
                0,   // spw
                0,   // spw
                228, // c - time (1)
                77,  // c - time (1)
                0, 0, 2,   // c - direction
                231, // c - heading
                4,   // c - speed
                14,  // c - zbyte
                217, // c - x (1)
                16,  // c - x (2)
                153, // c - y (1)
                5,   // c - y (2)
            ],
            |hlv: Hlv| {
                assert_eq!(hlv.reqi, RequestId(0));
                assert_eq!(hlv.plid, PlayerId(3));
                assert_eq!(hlv.time, Duration::from_millis(19940));
                assert!(matches!(hlv.hlvc, Hlvc::Wall));
            }
        );
    }
}
