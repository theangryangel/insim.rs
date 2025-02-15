use insim_core::binrw::{self, binrw};

use crate::identifiers::{PlayerId, RequestId};

#[binrw]
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[non_exhaustive]
// FIXME: Needs manual  BinRead and BinWrite impl
/// AI input type
pub enum AiInputType {
    /// Steering
    Msx(u32),

    /// Throttle
    Throttle(u32),

    /// Brake
    Brake(u32),

    /// Gear up
    Chup(u32),

    /// Gear down
    Chdn(u32),

    /// Ignition
    Ignition(u32),

    /// Extra lights
    ExtraLight(u32),

    /// Head lights
    HeadLights(u32),

    /// Siren
    Siren(u32),

    /// Honk
    Horn(u32),

    /// Flash
    Flash(u32),

    /// Clutch
    Clutch(u32),

    /// Handbrake
    Handbrake(u32),

    /// Indicators
    Indicators(u32),

    /// Gear
    Gear(u32),

    /// Look
    Look(u32),

    /// Pitspeed
    Pitspeed(u32),

    /// Disable Traction Control
    TcDisable(u32),

    /// Fogs rear
    FogRear(u32),

    /// Fogs front
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
// needs proper types
/// AI Input Control, value
pub struct AiInputVal {
    /// Input
    pub input: AiInputType,

    /// Duration
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
    // pub inputs: Vec<AiInputVal>,
}

impl_typical_with_request_id!(Aic);
