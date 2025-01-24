use bitflags::bitflags;
use insim_core::{
    binrw::{self, binrw},
    point::Point,
};

use crate::identifiers::{PlayerId, RequestId};

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct OsMain {
    pub angvel: (f32, f32, f32),

    pub heading: f32,

    pub pitch: f32,

    pub roll: f32,

    pub accel: Point<f32>,

    pub vel: Point<f32>,

    pub pos: Point<i32>,
}

bitflags! {
    /// Provides extended host information
    #[binrw]
    #[br(map = Self::from_bits_truncate)]
    #[bw(map = |&x: &Self| x.bits())]
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Default)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize))]
    pub struct AiFlags: u8 {
        /// Detect if engine running
        const IGNITION = (1 << 0);
        /// Upshift currently held
        const CHUP = (1 << 2);
        /// Downshift currently held
        const CHDN = (1 << 3);
    }
}

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// AI Info
pub struct Aii {
    /// Non-zero if the packet is a packet request or a reply to a request
    pub reqi: RequestId,

    /// Set to choose 16-bit
    pub plid: PlayerId,

    pub osmain: OsMain,

    pub flags: AiFlags,

    #[brw(pad_after = 2)]
    pub gear: u8,

    #[brw(pad_after = 8)]
    pub rpm: f32,

    #[brw(pad_after = 12)]
    pub showlights: u32,
}

impl_typical_with_request_id!(Aii);
