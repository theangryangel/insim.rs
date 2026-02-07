use bitflags::bitflags;

use crate::identifiers::RequestId;

#[derive(Debug, Default, Clone, insim_core::Decode, insim_core::Encode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
#[non_exhaustive]
/// Action type for [Oco].
pub enum OcoAction {
    #[default]
    /// Give up control of all lights
    LightsReset = 4,

    /// Use Data byte to set the bulbs
    LightsSet = 5,

    /// Give up control of the specified lights
    LightsUnset = 6,
}

#[derive(Debug, Default, Clone, insim_core::Decode, insim_core::Encode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
#[non_exhaustive]
/// Which start lights to manipulate.
pub enum OcoIndex {
    /// Layout lights 1
    AxoStartLights1 = 149,
    /// Layout lights 2
    AxoStartLights2 = 150,
    /// Layout lights 3
    AxoStartLights3 = 151,

    #[default]
    /// Main start lights
    MainLights = 240,
}

bitflags! {
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Default)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    /// Which bulbs to manipulate.
    pub struct OcoLights: u8 {
        /// Red1
        const RED1 = (1 << 0);
        /// Red2
        const RED2 = (1 << 1);
        /// Red3
        const RED3 = (1 << 2);
        /// Green
        const GREEN = (1 << 3);
    }
}

impl_bitflags_from_to_bytes!(OcoLights, u8);

#[derive(Debug, Clone, Default, insim_core::Decode, insim_core::Encode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Start light control packet.
///
/// - Controls main or layout start lights.
pub struct Oco {
    /// Request identifier echoed by replies.
    #[insim(pad_after = 1)]
    pub reqi: RequestId,

    /// Action to take.
    pub ocoaction: OcoAction,

    /// Which light group to control.
    pub index: OcoIndex,

    /// Optional identifier (layout light index).
    pub identifier: u8,

    /// Bulbs to enable/disable.
    pub data: OcoLights,
}

impl_typical_with_request_id!(Oco);

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_oco() {
        assert_from_to_bytes!(
            Oco,
            [
                0,   // reqi
                0,   // zero
                5,   // ocoaction
                149, // index
                35,  // identifier
                3,   // data
            ],
            |oco: Oco| {
                assert_eq!(oco.reqi, RequestId(0));
                assert!(matches!(oco.ocoaction, OcoAction::LightsSet));
                assert_eq!(oco.identifier, 35);
                assert_eq!(oco.data.bits(), 3);
            }
        );
    }
}
