use insim_core::binrw::{self, binrw};

use crate::identifiers::{PlayerId, RequestId};

#[derive(PartialEq, Eq, Clone, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[non_exhaustive]
// FIXME: Needs manual  BinRead and BinWrite impl
pub enum AiInputType {
    Msx(u32),

    Throttle(u32),

    Brake(u32),

    Chup(u32),

    Chdn(u32),

    Ignition(u32),

    ExtraLight(u32),

    HeadLights(u32),

    Siren(u32),

    Horn(u32),

    Flash(u32),

    Clutch(u32),

    Handbrake(u32),

    Indicators(u32),

    Gear(u32),

    Look(u32),

    Pitspeed(u32),

    TcDisable(u32),

    FogRear(u32),

    FogFront(u32),
}

impl Default for AiInputType {
    fn default() -> Self {
        Self::Msx(32768)
    }
}

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
// FIXME: Needs manual BinRead and BinWrite impl
pub struct AiInputVal {
    pub input: u8,
    pub time: u8,
}

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// AI Input Control
pub struct Aic {
    /// Non-zero if the packet is a packet request or a reply to a request
    pub reqi: RequestId,

    /// Set to choose 16-bit
    pub plid: PlayerId,

    // FIXME: Needs manual BinRead and BinWrite implementation
    pub inputs: Vec<AiInputVal>,
}

impl_typical_with_request_id!(Aic);
