use insim_core::{
    identifiers::{ConnectionId, RequestId},
    prelude::*,
    ser::Limit,
};

#[cfg(feature = "serde")]
use serde::Serialize;

use bitflags::bitflags;

bitflags! {
    #[derive(Default)]
    #[cfg_attr(feature = "serde", derive(Serialize))]
    pub struct PlcAllowedCars: u32 {
         const XF_GTI = (1 << 1);
         const XR_GT = (1 << 2);
         const XR_GT_TURBO = (1 << 3);
         const RB4 = (1 << 4);
         const FXO_TURBO = (1 << 5);
         const LX4 = (1 << 6);
         const LX6 = (1 << 7);
         const MRT5 = (1 << 8);
         const UF_1000 = (1 << 9);
         const RACEABOUT = (1 << 10);
         const FZ50 = (1 << 11);
         const FORMULA_XR = (1 << 12);
         const XF_GTR = (1 << 13);
         const UF_GTR = (1 << 14);
         const FORMULA_V8 = (1 << 15);
         const FXO_GTR = (1 << 16);
         const XR_GTR = (1 << 17);
         const FZ50_GTR = (1 << 18);
         const BWM_SAUBER_F1_06 = (1 << 19);
         const FORMULA_BMW_FB02 = (1 << 20);
    }
}

impl Decodable for PlcAllowedCars {
    fn decode(
        buf: &mut bytes::BytesMut,
        limit: Option<Limit>,
    ) -> Result<Self, insim_core::DecodableError>
    where
        Self: Default,
    {
        Ok(Self::from_bits_truncate(u32::decode(buf, None)?))
    }
}

impl Encodable for PlcAllowedCars {
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
/// Player Cars
pub struct Plc {
    #[insim(pad_bytes_after = "1")]
    pub reqi: RequestId,

    #[insim(pad_bytes_before = "3")]
    pub ucid: ConnectionId,

    pub allowed_cars: PlcAllowedCars,
}
