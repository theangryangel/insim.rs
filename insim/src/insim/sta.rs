use insim_core::{
    identifiers::{PlayerId, RequestId},
    binrw::{self, binrw},
    racelaps::RaceLaps,
    track::Track,
    wind::Wind,
};

#[cfg(feature = "serde")]
use serde::Serialize;

use bitflags::bitflags;

use super::CameraView;

#[binrw]
#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[repr(u8)]
#[brw(repr(u8))]
pub enum StaRacing {
    /// No race in progress
    #[default]
    No = 0,

    /// Race in progress
    Racing = 1,

    /// Qualifying
    Qualifying = 2,
}

bitflags! {
    /// Bitwise flags used within the [Sta] packet
    #[binrw]
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Default)]
    #[cfg_attr(feature = "serde", derive(Serialize))]
    #[br(map = Self::from_bits_truncate)]
    #[bw(map = |&x: &Self| x.bits())]
    pub struct StaFlags: u16 {
        /// In Game (or Multiplayer Replay)
        const GAME = (1 << 0);

        // In Singleplayer Relay
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
        const MULTI_SPEEDUP = (1 << 10);

        /// Windows mode
        const WINDOWED = (1 << 11);

        /// Muted
        const MUTED = (1 << 12);

        /// View override
        const VIEW_OVERRIDE = (1 << 13);

        /// Insim buttons visible
        const VISIBLE = (1 << 14);

        /// In a text entry dialog
        const TEXT_ENTRY = (1 << 15);
    }
}

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// State
pub struct Sta {
    #[brw(pad_after = 1)]
    pub reqi: RequestId,

    /// 1.0 is normal speed
    pub replayspeed: f32,

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
    pub raceinprog: StaRacing,

    pub qualmins: u8,
    #[brw(pad_after = 1)]
    pub racelaps: RaceLaps,
    pub serverstatus: u8, // serverstatus isn't an enum, unfortunately

    pub track: Track,
    pub weather: u8, // TODO: Weather is track dependant?!
    pub wind: Wind,
}

impl Sta {
    pub fn is_server_status_ok(&self) -> bool {
        self.serverstatus == 1
    }
}
