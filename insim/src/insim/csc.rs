use std::time::Duration;

use super::CarContact;
use crate::identifiers::{PlayerId, RequestId};

#[derive(Debug, Default, Clone, insim_core::Decode, insim_core::Encode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[repr(u8)]
#[non_exhaustive]
/// Used within the [Csc] packet to indicate the type of state change.
pub enum CscAction {
    #[default]
    /// Stopped
    Stop = 0,

    /// Started
    Start = 1,
}

#[derive(Debug, Clone, Default, insim_core::Decode, insim_core::Encode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Car State Changed - reports a change in a car's state (currently start or stop)
pub struct Csc {
    /// Non-zero if the packet is a packet request or a reply to a request
    pub reqi: RequestId,

    /// Unique player ID
    #[insim(pad_after = 1)]
    pub plid: PlayerId,

    /// Action that was taken
    #[insim(pad_after = 2)]
    pub cscaction: CscAction,

    /// Time since start (warning: this is looping)
    #[insim(duration(milliseconds = u32))]
    pub time: Duration,

    /// Any contact that may have happened
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
