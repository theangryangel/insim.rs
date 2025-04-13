use bitflags::bitflags;
use insim_core::{
    binrw::{self, binrw},
    track::Track,
    wind::Wind,
};

use super::{CameraView, RaceLaps};
use crate::identifiers::{PlayerId, RequestId};

#[binrw]
#[derive(Debug, Default, Clone, insim_macros::ReadWriteBuf)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[repr(u8)]
#[brw(repr(u8))]
#[non_exhaustive]
/// Game racing state
pub enum RaceInProgress {
    /// No race in progress
    #[default]
    No = 0,

    /// Race in progress
    Racing = 1,

    /// Qualifying
    Qualifying = 2,
}

bitflags! {
    #[binrw]
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Default)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize))]
    #[br(map = Self::from_bits_truncate)]
    #[bw(map = |&x: &Self| x.bits())]
    /// Describes the game state
    pub struct StaFlags: u16 {
        /// In Game (or Multiplayer Replay)
        const GAME = (1 << 0);

        /// In Singleplayer Replay
        const REPLAY = (1 << 1);

        /// Paused
        const PAUSE = (1 << 2);

        /// Shift+U mode
        const SHIFTU = (1 << 3);

        /// Dialog
        const DIALOG = (1 << 4);

        /// SHIFT+U follow
        const SHIFTU_FOLLOW = (1 << 5);

        /// SHIFT+U buttons hidden
        const SHIFTU_NO_OPT = (1 << 6);

        /// Showing 2D display
        const SHOW_2D = (1 << 7);

        /// Showing entry screen
        const FRONT_END = (1 << 8);

        /// Multiplayer mode
        const MULTI = (1 << 9);

        /// Multiplayer speedup
        const MPSPEEDUP = (1 << 10);

        /// Windows mode
        const WINDOWED = (1 << 11);

        /// Muted
        const SOUND_MUTE = (1 << 12);

        /// View override
        const VIEW_OVERRIDE = (1 << 13);

        /// Insim buttons visible
        const VISIBLE = (1 << 14);

        /// In a text entry dialog
        const TEXT_ENTRY = (1 << 15);
    }
}

generate_bitflag_helpers! {
    StaFlags,

    pub is_in_game => GAME,
    pub is_viewing_replay => REPLAY,
    pub is_shiftu => SHIFTU,
    pub is_shiftu_following => SHIFTU_FOLLOW,
    pub is_shiftu_buttons_hidden => SHIFTU_NO_OPT,
    pub is_multiplayer => MULTI,
    pub is_windowed => WINDOWED,
    pub is_muted => SOUND_MUTE,
    pub insim_buttons_visible => VISIBLE
}

impl_bitflags_from_to_bytes!(StaFlags, u16);

#[binrw]
#[derive(Debug, Clone, Default, insim_macros::ReadWriteBuf)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// State
pub struct Sta {
    #[brw(pad_after = 1)]
    #[read_write_buf(pad_after = 1)]
    /// Non-zero if the packet is a packet request or a reply to a request
    pub reqi: RequestId,

    /// 1.0 is normal speed
    pub replayspeed: f32,

    /// State of the game
    pub flags: StaFlags,

    /// Which type of camera is selected
    pub ingamecam: CameraView,

    /// Currently viewing player
    pub viewplid: PlayerId,

    /// Number of players in race
    pub nump: u8,

    /// Number of connections, including host
    pub numconns: u8,

    /// Number of finished or qualifying players
    pub numfinished: u8,

    /// Race status
    pub raceinprog: RaceInProgress,

    /// Qualifying minutes
    pub qualmins: u8,

    #[brw(pad_after = 1)]
    #[read_write_buf(pad_after = 1)]
    /// Number of laps
    pub racelaps: RaceLaps,

    /// Server status
    pub serverstatus: u8, // serverstatus isn't an enum, unfortunately

    /// The track
    pub track: Track,

    /// Weather conditions
    pub weather: u8, // TODO: Weather is track dependant?!

    /// Wind conditions
    pub wind: Wind,
}

impl Sta {
    /// Is server status healthy?
    pub fn is_server_status_ok(&self) -> bool {
        self.serverstatus == 1
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_sta() {
        assert_from_to_bytes!(
            Sta,
            [
                1,   // reqi
                0,   // zero
                0,   // replayspeed (1)
                0,   // replayspeed (2)
                128, // replayspeed (3)
                62,  // replayspeed (4)
                8,   // flags (1)
                0,   // flags (2)
                3,   // ingamecam
                4,   // viewplid
                32,  // nump
                47,  // numconns
                20,  // numfinished
                2,   // raceinprog
                60,  // qualmins
                12,  // racelaps
                0,   // sp2
                1,   // serverstatus
                b'B', b'L', b'2', b'R', 0, 0, //track
                1, // weather
                2, // wind
            ],
            |parsed: Sta| {
                assert_eq!(parsed.reqi, RequestId(1));
                assert_eq!(parsed.nump, 32);
                assert_eq!(parsed.numconns, 47);
                assert!(matches!(parsed.racelaps, RaceLaps::Laps(12)));
            }
        );
    }
}
