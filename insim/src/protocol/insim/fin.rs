use insim_core::{
    identifiers::{PlayerId, RequestId},
    prelude::*,
    ser::Limit,
};

#[cfg(feature = "serde")]
use serde::Serialize;

use bitflags::bitflags;

use super::PlayerFlags;

bitflags! {
    #[derive(Default)]
    #[cfg_attr(feature = "serde", derive(Serialize))]
    pub struct RaceResultFlags: u8 {
        const MENTIONED = (1 << 0);
        const CONFIRMED = (1 << 1);
        const PENALTY_DT = (1 << 2);
        const PENALTY_SG = (1 << 3);
        const PENALTY_30 = (1 << 4);
        const PENALTY_45 = (1 << 5);
        const NO_PIT = (1 << 6);
    }
}

impl Encodable for RaceResultFlags {
    fn encode(&self, buf: &mut bytes::BytesMut, limit: Option<Limit>) -> Result<(), insim_core::EncodableError>
    where
        Self: Sized,
    {
        self.bits().encode(buf, limit)?;
        Ok(())
    }
}

impl Decodable for RaceResultFlags {
    fn decode(
        buf: &mut bytes::BytesMut,
        limit: Option<Limit>
    ) -> Result<Self, insim_core::DecodableError> {

        Ok(Self::from_bits_truncate(u8::decode(buf, limit)?))
    }
}

impl RaceResultFlags {
    /// Was the player disqualified for any reason?
    pub fn disqualified(&self) -> bool {
        self.contains(RaceResultFlags::PENALTY_DT)
            || self.contains(RaceResultFlags::PENALTY_SG)
            || self.contains(RaceResultFlags::NO_PIT)
    }

    /// Did the player receive a penalty for any reason?
    pub fn time_penalty(&self) -> bool {
        self.contains(RaceResultFlags::PENALTY_30) || self.contains(RaceResultFlags::PENALTY_45)
    }
}

#[derive(Debug, InsimEncode, InsimDecode, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Provisional finish notification: This is not a final result, you should use the [Res](super::Res) packet for this instead.
pub struct Fin {
    pub reqi: RequestId,

    pub plid: PlayerId,

    pub ttime: u32,

    #[insim(pad_bytes_after = "1")]
    pub btime: u32,

    pub numstops: u8,

    #[insim(pad_bytes_after = "1")]
    pub confirm: RaceResultFlags,

    pub lapsdone: u16,

    pub flags: PlayerFlags,
}
