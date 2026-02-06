use std::time::Duration;

use super::{CarContact, ObjectInfo};
use crate::identifiers::{PlayerId, RequestId};

#[derive(Debug, Default, Clone, insim_core::Decode, insim_core::Encode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
#[non_exhaustive]
/// Action reported by [Uco].
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

#[derive(Debug, Clone, insim_core::Decode, insim_core::Encode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// User control object event.
///
/// - Reports crossing a checkpoint or entering/leaving a circle.
pub struct Uco {
    /// Request identifier echoed by replies.
    pub reqi: RequestId,

    /// Player the event applies to.
    #[insim(pad_after = 1)]
    pub plid: PlayerId,

    /// Event type.
    #[insim(pad_after = 2)]
    pub ucoaction: UcoAction,

    /// Time since session start (wraps periodically).
    #[insim(duration = u32)]
    pub time: Duration,

    /// Contact details, if relevant.
    pub c: CarContact,

    /// Checkpoint/circle object details.
    pub info: ObjectInfo,
}

#[cfg(test)]
mod test {
    use insim_core::object::{ObjectInfo, insim::InsimCircle};

    use super::*;

    #[test]
    fn test_uco() {
        assert_from_to_bytes!(
            Uco,
            [
                0,   // reqi
                2,   // plid
                0,   // sp0
                1,   // ucoaction
                0,   // sp2
                0,   // sp3
                8,   // time (1)
                80,  // time (2)
                2,   // time (3)
                0,   // time (4)
                0,   // c - direction
                126, // c - heading
                8,   // c - speed
                10,  // c - zbyte
                198, // c - x (1)
                254, // c - x (2)
                40,  // c - y (1)
                250, // c - y (2)
                232, // info - x (1)
                254, // info - x (2)
                207, // info - y (1)
                249, // info - y (2)
                8,   // info - zbyte
                0,   // info - flags
                253, // info - index
                1,   // info - heading
            ],
            |parsed: Uco| {
                assert_eq!(
                    parsed.info.position().xyz_metres(),
                    (
                        -17.5,    // -280 / 16,
                        -99.0625, // -1585 / 16,
                        2.0,      // 8.0 / 4
                    )
                );
                assert!(matches!(
                    parsed.info,
                    ObjectInfo::InsimCircle(InsimCircle {
                        index: 1,
                        floating: false,
                        ..
                    })
                ));
                assert!(matches!(parsed.ucoaction, UcoAction::CircleLeave));
                assert_eq!(parsed.time, Duration::from_millis(151560));
                assert_eq!(parsed.c.speed.to_meters_per_sec() as u8, 8);
                assert_eq!(parsed.c.x, -314);
                assert_eq!(parsed.c.y, -1496);
                assert_eq!(parsed.c.z, 10);
            }
        );
    }
}
