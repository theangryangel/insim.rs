use insim_core::{
    identifiers::{PlayerId, RequestId},
    prelude::*,
    ser::Limit,
};

#[cfg(feature = "serde")]
use serde::Serialize;

use bitflags::bitflags;

bitflags! {
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Default)]
    #[cfg_attr(feature = "serde", derive(Serialize))]
    pub struct ObhFlags: u8 {
        const LAYOUT = (1 << 0);
        const CAN_MOVE = (1 << 1);
        const WAS_MOVING = (1 << 2);
        const ON_SPOT = (1 << 3);
    }
}

impl Encodable for ObhFlags {
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

impl Decodable for ObhFlags {
    fn decode(
        buf: &mut bytes::BytesMut,
        limit: Option<Limit>,
    ) -> Result<Self, insim_core::DecodableError>
    where
        Self: Sized,
    {
        Ok(Self::from_bits_truncate(u8::decode(buf, limit)?))
    }
}

#[derive(Debug, InsimEncode, InsimDecode, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct CarContact {
    pub direction: u8,
    pub heading: u8,
    pub speed: u8,
    pub z: u8,
    pub x: i16,
    pub y: i16,
}

#[derive(Debug, InsimEncode, InsimDecode, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Object Hit
pub struct Obh {
    pub reqi: RequestId,
    pub plid: PlayerId,

    pub spclose: u16,
    pub time: u16,

    pub info: CarContact,

    pub x: i16,
    pub y: i16,

    #[insim(pad_bytes_after = "1")]
    pub z: u8,
    pub index: u8,

    pub flags: ObhFlags,
}
