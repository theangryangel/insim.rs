use std::time::Duration;

use crate::identifiers::RequestId;

#[derive(Debug, Default, Clone, insim_core::Decode, insim_core::Encode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "python", pyo3::prelude::pyclass)]
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
    #[cfg_attr(feature = "python", pyo3::prelude::pyclass)]
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

#[derive(Debug, Clone, Default, insim_core::Decode, insim_core::Encode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "python", pyo3::prelude::pyclass)]
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
    #[insim(pad_after = 1)]
    pub options: RipOptions,

    /// Request: destination / Reply: position
    #[insim(duration(milliseconds = u32))]
    pub ctime: Duration,

    /// Request: zero / reply: replay length
    #[insim(duration(milliseconds = u32))]
    pub ttime: Duration,

    /// Zero or replay name
    #[insim(codepage(length = 64, trailing_nul = true))]
    pub rname: String,
}

impl_typical_with_request_id!(Rip);

#[cfg(test)]
mod test {
    use bytes::{BufMut, BytesMut};

    use super::*;

    #[test]
    fn test_rip() {
        let mut data = BytesMut::new();
        data.extend_from_slice(&[
            2,   // reqi
            0,   // error
            1,   // mpr
            1,   // paused
            6,   // options
            0,   // sp3
            116, // ctime (1)
            41,  // ctime (2)
            2,   // ctime (3)
            0,   // ctime (4)
            0,   // ttime (1)
            0,   // ttime (2)
            0,   // ttime (3)
            0,   // ttime (4)
        ]);
        data.extend_from_slice(b"name_of_thing");
        data.put_bytes(0, 64 - 13);

        assert_from_to_bytes!(Rip, data.as_ref(), |parsed: Rip| {
            assert_eq!(parsed.reqi, RequestId(2));
            assert_eq!(parsed.mpr, true);
            assert_eq!(parsed.paused, true);
            assert_eq!(parsed.ctime, Duration::from_millis(141684));
            assert_eq!(parsed.rname, "name_of_thing");
        });
    }
}
