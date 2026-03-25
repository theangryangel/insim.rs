use std::convert::From;

use bytes::{Buf, BufMut};
use insim_core::{Decode, DecodeContext, Encode, EncodeContext};

/// Handles the rules around how RaceLaps are described within Insim automatically for you.
#[derive(Debug, Default, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum RaceLaps {
    /// This is an untimed session
    Untimed,
    /// This is a practise session
    #[default]
    Practice,
    /// This is a fixed number of laps
    Laps(usize),
    /// This is a time-based event
    Hours(usize),
}

impl From<u8> for RaceLaps {
    fn from(value: u8) -> Self {
        let value = value as usize;
        match value {
            255 => RaceLaps::Untimed,
            0 => RaceLaps::Practice,
            1..=99 => RaceLaps::Laps(value),
            100..=190 => RaceLaps::Laps((value - 100) * 10 + 100),
            191..=238 => RaceLaps::Hours(value - 190),
            _ => RaceLaps::Practice,
        }
    }
}

impl From<RaceLaps> for u8 {
    fn from(item: RaceLaps) -> u8 {
        let data = match item {
            RaceLaps::Untimed => 255,
            RaceLaps::Practice => 0,
            RaceLaps::Laps(data) => {
                match data {
                    1..=99 => data,
                    100..=1000 => ((data - 100) / 10) + 100,
                    _ => 0, // if it's an invalid structure we're going to push it into practice
                }
            },
            RaceLaps::Hours(data) => data + 190,
        };

        data as u8
    }
}

impl Decode for RaceLaps {
    const PRIMITIVE: bool = true;
    fn decode(ctx: &mut DecodeContext) -> Result<Self, insim_core::DecodeError> {
        let val = ctx.buf.get_u8();
        Ok(val.into())
    }
}

impl Encode for RaceLaps {
    const PRIMITIVE: bool = true;
    fn encode(&self, ctx: &mut EncodeContext) -> Result<(), insim_core::EncodeError> {
        let val = u8::from(*self);
        ctx.buf.put_u8(val);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn untimed() {
        let data = RaceLaps::Untimed;
        assert_eq!(Into::<u8>::into(data), 255);
    }

    #[test]
    fn as_practise() {
        let data = RaceLaps::Practice;
        assert_eq!(Into::<u8>::into(data), 0);
    }

    #[test]
    fn as_simple_laps() {
        assert_eq!(Into::<u8>::into(RaceLaps::Laps(1)), 1);
        assert_eq!(1, Into::<u8>::into(RaceLaps::Laps(1)));

        assert_eq!(Into::<u8>::into(RaceLaps::Laps(99)), 99);
        assert_eq!(99, Into::<u8>::into(RaceLaps::Laps(99)));
    }

    #[test]
    fn as_complex_laps() {
        assert_eq!(Into::<u8>::into(RaceLaps::Laps(199)), 109);
        assert_eq!(109, Into::<u8>::into(RaceLaps::Laps(199)));
    }

    #[test]
    fn as_hours() {
        assert_eq!(Into::<u8>::into(RaceLaps::Hours(1)), 191);
        assert_eq!(191, Into::<u8>::into(RaceLaps::Hours(1)));
    }
}
