use insim_core::{identifiers::RequestId, prelude::*};

#[cfg(feature = "serde")]
use serde::Serialize;

use bitflags::bitflags;

#[derive(Debug, InsimEncode, InsimDecode, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[repr(u8)]
pub enum OcoAction {
    LightsReset = 4,

    LightsSet = 5,

    LightsUnset = 6,
}

impl Default for OcoAction {
    fn default() -> Self {
        OcoAction::LightsReset
    }
}

#[derive(Debug, InsimEncode, InsimDecode, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[repr(u8)]
pub enum OcoIndex {
    AxoStartLights = 149,

    MainLights = 240,
}

impl Default for OcoIndex {
    fn default() -> Self {
        OcoIndex::MainLights
    }
}

bitflags! {
    #[derive(Default)]
    #[cfg_attr(feature = "serde", derive(Serialize))]
    pub struct OcoLights: u8 {
        const RED1 = (1 << 0);
        const RED2 = (1 << 1);
        const RED3 = (1 << 2);
        const GREEN = (1 << 3);
    }
}

impl Encodable for OcoLights {
    fn encode(&self, buf: &mut bytes::BytesMut) -> Result<(), insim_core::EncodableError>
    where
        Self: Sized,
    {
        self.bits().encode(buf)?;
        Ok(())
    }
}

impl Decodable for OcoLights {
    fn decode(
        buf: &mut bytes::BytesMut,
        count: Option<usize>,
    ) -> Result<Self, insim_core::DecodableError>
    where
        Self: Sized,
    {
        Ok(Self::from_bits_truncate(u8::decode(buf, count)?))
    }
}

#[derive(Debug, InsimEncode, InsimDecode, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Object Control
/// Used to switch start lights
pub struct Oco {
    #[insim(pad_bytes_after = "1")]
    pub reqi: RequestId,

    pub action: OcoAction,

    pub index: OcoIndex,

    pub identifer: u8,

    pub lights: OcoLights,
}
