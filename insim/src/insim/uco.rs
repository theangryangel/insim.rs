use std::time::Duration;

use super::{CarContact, ObjectInfo};
use crate::identifiers::{PlayerId, RequestId};

#[derive(Debug, Default, Clone, insim_core::Decode, insim_core::Encode)]
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

#[derive(Debug, Clone, Default, insim_core::Decode, insim_core::Encode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// User Control Object - reports crossing an InSim checkpoint / entering an InSim circle (from layout)
pub struct Uco {
    /// Non-zero if the packet is a packet request or a reply to a request
    pub reqi: RequestId,

    /// Player's unique ID that this report corresponds to.
    #[insim(pad_after = 1)]
    pub plid: PlayerId,

    /// What happened
    #[insim(pad_after = 2)]
    pub ucoaction: UcoAction,

    /// When this happened
    #[insim(duration = u32)]
    pub time: Duration,

    /// Was there any car contact?
    pub c: CarContact,

    /// Info about the checkpoint or circle (see below)
    pub info: ObjectInfo,
}

#[cfg(test)]
mod test {
    use insim_core::object::{ObjectPosition, control};

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
                24,  // info - flags
                253, // info - index
                1,   // info - heading
            ],
            |parsed: Uco| {
                let expected = ObjectInfo::InsimCircle(control::InsimCircle {
                    xyz: ObjectPosition {
                        x: -280,
                        y: -1585,
                        z: 8,
                    },
                    flags: 24,
                    index: 1,
                });
                assert_eq!(parsed.info, expected);
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
