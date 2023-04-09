use insim_core::{
    identifiers::{PlayerId, RequestId},
    prelude::*,
    ser::Limit,
    track::Track,
    wind::Wind,
};

#[cfg(feature = "serde")]
use serde::Serialize;

use bitflags::bitflags;

use super::CameraView;

bitflags! {
    /// Bitwise flags used within the [Sta] packet
    #[derive(Default)]
    #[cfg_attr(feature = "serde", derive(Serialize))]
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

impl Decodable for StaFlags {
    fn decode(
        buf: &mut bytes::BytesMut,
        limit: Option<Limit>,
    ) -> Result<Self, insim_core::DecodableError>
    where
        Self: Default,
    {
        Ok(Self::from_bits_truncate(u16::decode(buf, limit)?))
    }
}

impl Encodable for StaFlags {
    fn encode(
        &self,
        buf: &mut bytes::BytesMut,
        limit: Option<Limit>,
    ) -> Result<(), insim_core::EncodableError>
    where
        Self: Sized,
    {
        self.bits().encode(buf, limit)?;
        Ok(())
    }
}

#[derive(Debug, InsimEncode, InsimDecode, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// State
pub struct Sta {
    #[insim(pad_bytes_after = "1")]
    pub reqi: RequestId,

    pub replayspeed: f32,

    pub flags: StaFlags,
    pub ingamecam: CameraView,
    pub viewplid: PlayerId,

    pub nump: u8,
    pub numconns: u8,
    pub numfinished: u8,
    pub raceinprog: u8,

    pub qualmins: u8,
    #[insim(pad_bytes_after = "1")]
    pub racelaps: u8,
    pub serverstatus: u8, // serverstatus isn't an enum, unfortunately

    pub track: Track,
    pub weather: u8, // Weather is track dependant?!
    pub wind: Wind,
}

impl Sta {
    pub fn is_server_status_ok(&self) -> bool {
        self.serverstatus == 1
    }
}
