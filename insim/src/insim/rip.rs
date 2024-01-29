use std::time::Duration;

use insim_core::{
    binrw::{self, binrw},
    duration::{binrw_parse_duration, binrw_write_duration},
    string::{binrw_parse_codepage_string, binrw_write_codepage_string},
};

use crate::identifiers::RequestId;

#[binrw]
#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[repr(u8)]
#[brw(repr(u8))]
#[non_exhaustive]
/// Replay Information Error
pub enum RipError {
    #[default]
    /// Ok - No error!
    Ok = 0,

    /// Already at the destination
    Already = 1,

    /// Can't run a replay - dedicated host
    Dedicated = 2,

    /// Can't start a replay - not in a suitable mode
    WrongMode = 3,

    /// RName is zero but no replay is currently loaded
    NotReplay = 4,

    /// Replay is corrupt
    Corrupted = 5,

    /// Could not find replay
    NotFound = 6,

    /// Could not load replay.
    Unloadable = 7,

    /// Destination is beyond replay length
    DestOOB = 8,

    /// Unknown error found starting replay
    Unknown = 9,

    /// Replay search was terminated by user
    User = 10,

    /// Can't reach destination - SPR is out of sync
    OOS = 11,
}

bitflags::bitflags! {
    /// Bitwise flags used within the [Rip] packet
    #[binrw]
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Default)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize))]
    #[br(map = Self::from_bits_truncate)]
    #[bw(map = |&x: &Self| x.bits())]
    pub struct RipOptions: u8 {
        /// Replay will loop
        const LOOP = (1 << 0);

        /// Download missing skins
        const SKINS = (1 << 1);

        /// Use full physics
        const FULL_PHYS = (1 << 2);
    }
}

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Replay Information
pub struct Rip {
    /// Non-zero if the packet is a packet request or a reply to a request
    pub reqi: RequestId,

    /// 0 or 1 = OK
    pub error: RipError,

    /// Multiplayer replay?
    #[br(map = |x: u8| x != 0)]
    #[bw(map = |&x| x as u8)]
    pub mpr: bool,

    /// Paused playback
    #[br(map = |x: u8| x != 0)]
    #[bw(map = |&x| x as u8)]
    pub paused: bool,

    /// Misc options. See [RipOptions].
    #[brw(pad_after = 1)]
    pub options: RipOptions,

    /// Request: destination / Reply: position
    #[br(parse_with = binrw_parse_duration::<u32, 1, _>)]
    #[bw(write_with = binrw_write_duration::<u32, 1, _>)]
    pub ctime: Duration,

    /// Request: zero / reply: replay length
    #[br(parse_with = binrw_parse_duration::<u32, 1, _>)]
    #[bw(write_with = binrw_write_duration::<u32, 1, _>)]
    pub ttime: Duration,

    /// Zero or replay nam
    #[bw(write_with = binrw_write_codepage_string::<64, _>)]
    #[br(parse_with = binrw_parse_codepage_string::<64, _>)]
    pub rname: String,
}
