use bitflags::bitflags;
use insim_core::{Decode, Encode, track::Track, wind::Wind};

use super::RaceLaps;
use crate::identifiers::RequestId;

bitflags! {
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Default)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    /// Server and race configuration flags.
    pub struct RaceFlags: u16 {
        /// Can vote
        const CAN_VOTE = (1 << 0);
        /// Can select
        const CAN_SELECT = (1 << 1);
        /// Can join mid-race
        const MID_RACE = (1 << 5);
        /// Mandatory pit stop
        const MUST_PIT = (1 << 6);
        /// Car reset allowed
        const CAN_RESET = (1 << 7);
        /// Force cockpit view
        const FCV = (1 << 8);
        /// "Cruise" (no race, just free drive)
        const CRUISE = (1 << 9);
        /// Show fuel
        const SHOW_FUEL = (1 << 10);
        /// Can Refuel
        const CAN_REFUEL = (1 << 11);
        /// Allow mods?
        const ALLOW_MODS = (1 << 12);
        /// Allow unapproved mods?
        const UNAPPROVED = (1 << 13);
        /// Team arrows?
        const TEAM_ARROWS = (1 << 14);
        /// No Floodlights
        const NO_FLOOD = (1 << 15);
    }
}

generate_bitflag_helpers!(
    RaceFlags,
    pub can_vote => CAN_VOTE,
    pub can_select => CAN_SELECT,
    pub can_mid_race_join => MID_RACE,
    pub can_reset => CAN_RESET,
    pub forces_cockpit_view => FCV,
    pub is_cruise => CRUISE,
    pub show_fuel => SHOW_FUEL,
    pub can_refuel => CAN_REFUEL,
    pub allow_mods => ALLOW_MODS,
    pub allow_unapproved_mods => UNAPPROVED,
    pub team_arrows => TEAM_ARROWS,
    pub no_flood_lights => NO_FLOOD
);

impl_bitflags_from_to_bytes!(RaceFlags, u16);

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Lap timing configuration for the current session.
pub enum LapTimingInfo {
    /// Standard lap timing with a given number of checkpoints.
    Standard(u8), // 0x40 + checkpoint count (0–3)
    /// Custom timing with user-placed checkpoints.
    Custom(u8), // 0x80 + checkpoint count (0–3)
    #[default]
    /// No lap timing (open configuration without checkpoints).
    None, // 0xC0
}

impl LapTimingInfo {
    /// Read from u8
    pub fn from_u8(val: u8) -> Self {
        match val & 0xC0 {
            0x40 => LapTimingInfo::Standard(val & 0x03),
            0x80 => LapTimingInfo::Custom(val & 0x03),
            0xC0 => LapTimingInfo::None,
            _ => LapTimingInfo::None,
        }
    }

    /// Convert to u8
    pub fn to_u8(self) -> u8 {
        match self {
            LapTimingInfo::Standard(checkpoints) => 0x40 | (checkpoints & 0x03),
            LapTimingInfo::Custom(checkpoints) => 0x80 | (checkpoints & 0x03),
            LapTimingInfo::None => 0xC0,
        }
    }
}

impl Decode for LapTimingInfo {
    fn decode(buf: &mut bytes::Bytes) -> Result<Self, insim_core::DecodeError> {
        Ok(LapTimingInfo::from_u8(
            u8::decode(buf).map_err(|e| e.nested().context("LapTimingInfo::value"))?,
        ))
    }
}

impl Encode for LapTimingInfo {
    fn encode(&self, buf: &mut bytes::BytesMut) -> Result<(), insim_core::EncodeError> {
        self.to_u8()
            .encode(buf)
            .map_err(|e| e.nested().context("LapTimingInfo::value"))
    }
}

#[derive(Debug, Clone, Default, insim_core::Decode, insim_core::Encode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Race start information and session configuration.
///
/// - Sent when a race or qualifying session begins.
/// - Can be requested via [`TinyType::Rst`](crate::insim::TinyType::Rst).
pub struct Rst {
    /// Request identifier echoed by replies.
    #[insim(pad_after = 1)]
    pub reqi: RequestId,

    /// Race laps or session duration.
    pub racelaps: RaceLaps,

    /// Qualifying minutes (0 if racing).
    pub qualmins: u8,

    /// Number of players in the race.
    pub nump: u8,

    /// Lap timing configuration.
    pub timing: LapTimingInfo,

    /// Track identifier.
    pub track: Track,

    /// Weather identifier.
    pub weather: u8,

    /// Wind conditions.
    pub wind: Wind,

    /// Host and race settings.
    pub flags: RaceFlags,

    /// Total number of nodes in the path.
    pub numnodes: u16,

    /// Node index for the finish line.
    pub finish: u16,

    /// Node index for split 1.
    pub split1: u16,

    /// Node index for split 2.
    pub split2: u16,

    /// Node index for split 3.
    pub split3: u16,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_rst() {
        assert_from_to_bytes!(
            Rst,
            [
                1,  // reqi
                0,  // zero
                10, // racelaps
                45, // qualmins
                12, // nump
                66, // timing
                b'B', b'L', b'1', 0, 0, 0,   // track
                2,   // weather
                2,   // wind
                2,   // flags (1)
                1,   // flags (2)
                146, // numnodes (1)
                1,   // numnodes (2)
                109, // finish (1)
                1,   // finish (2)
                95,  // split1 (1)
                0,   // split1 (2)
                255, // split2 (1)
                0,   // split2 (2)
                255, // split3 (1)
                255, // split3 (2)
            ],
            |parsed: Rst| {
                assert_eq!(parsed.reqi, RequestId(1));
                assert_eq!(parsed.nump, 12);
                assert_eq!(parsed.numnodes, 402);
                assert!(matches!(parsed.racelaps, RaceLaps::Laps(10)));
                assert_eq!(parsed.qualmins, 45);
                assert!(matches!(parsed.wind, Wind::Strong));
            }
        )
    }
}
