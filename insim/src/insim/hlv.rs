use std::time::Duration;

use insim_core::{
    binrw::{self, binrw},
    duration::{binrw_parse_duration, binrw_write_duration},
};

use super::CarContact;
use crate::identifiers::{PlayerId, RequestId};

#[binrw]
#[derive(Debug, Default, Clone, insim_macros::ReadWriteBuf)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[repr(u8)]
#[brw(repr(u8))]
#[non_exhaustive]
/// Used within [Hlv] to indicate the hotlap validity failure reason.
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

#[binrw]
#[derive(Debug, Clone, Default, insim_macros::ReadWriteBuf)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Reports incidents that would violate Hot Lap Validity checks.
pub struct Hlv {
    /// Non-zero if the packet is a packet request or a reply to a request
    pub reqi: RequestId,

    /// Unique ID of player
    pub plid: PlayerId,

    /// How did we invalidate this hotlap? See [Hlvc].
    #[brw(pad_after = 1)]
    #[read_write_buf(pad_after = 1)]
    pub hlvc: Hlvc,

    #[br(parse_with = binrw_parse_duration::<u16, 10, _>)]
    #[bw(write_with = binrw_write_duration::<u16, 10, _>)]
    #[read_write_buf(duration(centiseconds = u16))]
    /// When the violation occurred. Warning: this is looping.
    pub time: Duration,

    /// Additional contact information. See [CarContact].
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
                202, // c - time (1)
                7,   // c - time (1)
                2,   // c - direction
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
