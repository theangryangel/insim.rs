use insim_core::{identifiers::RequestId, prelude::*, ser::Limit, track::Track, wind::Wind};

#[cfg(feature = "serde")]
use serde::Serialize;

use bitflags::bitflags;

bitflags! {
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Default)]
    #[cfg_attr(feature = "serde", derive(Serialize))]
    pub struct HostFacts: u16 {
         const CAN_VOTE = (1 << 0);
         const CAN_SELECT = (1 << 1);
         const MID_RACE_JOIN = (1 << 2);
         const MUST_PIT = (1 << 3);
         const CAN_RESET = (1 << 4);
         const FORCE_DRIVER_VIEW = (1 << 5);
         const CRUISE = (1 << 6);
    }
}

impl Decodable for HostFacts {
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

impl Encodable for HostFacts {
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
/// Race Start
pub struct Rst {
    #[insim(pad_bytes_after = "1")]
    pub reqi: RequestId,

    /// Racing laps, 0 if qualifying
    pub racelaps: u8,
    /// Qualifying minutes, 0 if racing
    pub qualmins: u8,

    pub nump: u8,
    pub timing: u8,

    pub track: Track,
    pub weather: u8,
    pub wind: Wind,

    pub flags: HostFacts,
    pub numnodes: u16,
    pub finish: u16,
    pub split1: u16,
    pub split2: u16,
    pub split3: u16,
}
