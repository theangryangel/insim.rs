use std::time::Duration;

use crate::identifiers::RequestId;

#[derive(Debug, Default, Clone, insim_macros::ReadWriteBuf)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[repr(u8)]
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
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Default)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize))]
    pub struct RipOptions: u8 {
        /// Replay will loop
        const LOOP = (1 << 0);

        /// Download missing skins
        const SKINS = (1 << 1);

        /// Use full physics
        const FULL_PHYS = (1 << 2);
    }
}

generate_bitflag_helpers! {
    RipOptions,

    pub is_looping => LOOP,
    pub missing_skin_download_enabled => SKINS,
    pub is_full_physics_simulation => FULL_PHYS
}

impl_bitflags_from_to_bytes!(RipOptions, u8);

#[derive(Debug, Clone, Default, insim_macros::ReadWriteBuf)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Replay Information
pub struct Rip {
    /// Non-zero if the packet is a packet request or a reply to a request
    pub reqi: RequestId,

    /// 0 or 1 = OK
    pub error: RipError,

    /// Multiplayer replay?
    pub mpr: bool,

    /// Paused playback
    pub paused: bool,

    /// Misc options. See [RipOptions].
    #[read_write_buf(pad_after = 1)]
    pub options: RipOptions,

    /// Request: destination / Reply: position
    #[read_write_buf(duration(milliseconds = u32))]
    pub ctime: Duration,

    /// Request: zero / reply: replay length
    #[read_write_buf(duration(milliseconds = u32))]
    pub ttime: Duration,

    /// Zero or replay name
    // FIXME: Not a codepage. probably not an ascii string either.. It's probably a wchar_t?
    #[read_write_buf(ascii(length = 64))]
    pub rname: String,
}

impl_typical_with_request_id!(Rip);
