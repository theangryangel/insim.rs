use insim_core::{
    identifiers::{PlayerId, RequestId},
    prelude::*,
    ser::Limit,
};

#[cfg(feature = "serde")]
use serde::Serialize;

use bitflags::bitflags;

bitflags! {
    // *_VALID variation means this was cleared
    #[derive(Default)]
    #[cfg_attr(feature = "serde", derive(Serialize))]
    pub struct PenaltyInfo: u8 {
        const DRIVE_THRU = (1 << 0);
        const DRIVE_THRU_VALID = (1 << 1);
        const STOP_GO = (1 << 2);
        const STOP_GO_VALID = (1 << 3);
        const SECS_30 = (1 << 4);
        const SECS_45 = (1 << 5);
    }
}

impl Encodable for PenaltyInfo {
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

impl Decodable for PenaltyInfo {
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

#[derive(Debug, Default, InsimEncode, InsimDecode, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[repr(u8)]
pub enum PenaltyReason {
    /// Unknown or cleared penalty
    #[default]
    None = 0,

    /// Penalty given by admin
    Admin = 1,

    /// Driving wrong way
    WrongWay = 2,

    /// False start
    FalseStart = 3,

    /// Speeding in pit lane
    Speeding = 4,

    /// Stop-go in pit stop too short
    StopShort = 5,

    /// Compulsory stop is too late
    StopLate = 6,
}

#[derive(Debug, InsimEncode, InsimDecode, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Penalty
pub struct Pen {
    pub reqi: RequestId,

    pub plid: PlayerId,

    pub oldpen: PenaltyInfo,

    pub newpen: PenaltyInfo,

    #[insim(pad_bytes_after = "1")]
    pub reason: PenaltyReason,
}
