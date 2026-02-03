use std::time::Duration;

use insim_core::{Decode, Encode};

use super::{PenaltyInfo, PlayerFlags};
use crate::identifiers::{PlayerId, RequestId};

#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[derive(Debug, Default, Clone)]
#[non_exhaustive]
/// Fuel percentage with the `/showfuel` multiplier applied.
pub enum Fuel200 {
    /// Double fuel percent
    Percentage(u8),

    /// Fuel cannot be reported, /showfuel=no
    #[default]
    No,
}

impl Decode for Fuel200 {
    fn decode(buf: &mut bytes::Bytes) -> Result<Self, insim_core::DecodeError> {
        let data = u8::decode(buf).map_err(|e| e.nested().context("Fuel200::data"))?;

        if data == 255 {
            Ok(Self::No)
        } else {
            Ok(Self::Percentage(data))
        }
    }
}

impl Encode for Fuel200 {
    fn encode(&self, buf: &mut bytes::BytesMut) -> Result<(), insim_core::EncodeError> {
        let data = match self {
            Self::Percentage(data) => *data,
            Self::No => 255_u8,
        };

        data.encode(buf)
            .map_err(|e| e.nested().context("Fuel200::data"))
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[derive(Debug, Default, Clone)]
#[non_exhaustive]
/// Fuel percentage for pit stops.
pub enum Fuel {
    /// Double fuel percent
    Percentage(u8),

    /// Fuel cannot be reported, /showfuel=no
    #[default]
    No,
}

impl Decode for Fuel {
    fn decode(buf: &mut bytes::Bytes) -> Result<Self, insim_core::DecodeError> {
        let data = u8::decode(buf).map_err(|e| e.nested().context("Fuel::data"))?;
        if data == 255 {
            Ok(Self::No)
        } else {
            Ok(Self::Percentage(data))
        }
    }
}

impl Encode for Fuel {
    fn encode(&self, buf: &mut bytes::BytesMut) -> Result<(), insim_core::EncodeError> {
        let data = match self {
            Self::Percentage(data) => *data,
            Self::No => 255_u8,
        };
        data.encode(buf)
            .map_err(|e| e.nested().context("Fuel::data"))?;
        Ok(())
    }
}

#[derive(Debug, Clone, Default, insim_core::Decode, insim_core::Encode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Lap timing information for a player.
///
/// - Sent when a lap is completed.
pub struct Lap {
    /// Request identifier echoed by replies.
    pub reqi: RequestId,

    /// Player that completed the lap.
    pub plid: PlayerId,

    #[insim(duration = u32)]
    /// Lap time.
    pub ltime: Duration, // lap time (ms)

    #[insim(duration = u32)]
    /// Total elapsed time since session start.
    pub etime: Duration,

    /// Number of laps completed.
    pub lapsdone: u16,

    /// Player flags (help settings).
    #[insim(pad_after = 1)]
    pub flags: PlayerFlags,

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
    fn test_lap() {
        assert_from_to_bytes!(
            Lap,
            [
                0,  // reqi
                2,  // plid
                4,  // ltime (1)
                0,  // ltime (2)
                0,  // ltime (3)
                1,  // ltime (4)
                64, // etime (1)
                0,  // etime (2)
                1,  // etime (3)
                0,  // etime (4)
                1,  // lapsdone (1)
                2,  // lapsdone (2)
                64, // flags (1)
                2,  // flags (2)
                0,  // sp0
                6,  // penalty
                3,  // numstops
                40, // fuel200
            ],
            |lap: Lap| {
                assert_eq!(lap.plid, PlayerId(2));
                assert_eq!(lap.ltime, Duration::from_millis(16777220));
                assert_eq!(lap.etime, Duration::from_millis(65600));
                assert_eq!(lap.lapsdone, 513);
                assert_eq!(lap.numstops, 3);
                assert!(matches!(lap.fuel200, Fuel200::Percentage(40)));
            }
        )
    }
}
