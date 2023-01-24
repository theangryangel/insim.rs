use insim_core::{identifiers::RequestId, prelude::*};

#[cfg(feature = "serde")]
use serde::Serialize;

use bitflags::bitflags;

use crate::track::Track;

#[derive(Debug, InsimEncode, InsimDecode, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[repr(u8)]
pub enum Wind {
    None = 0,
    Weak = 1,
    Strong = 2,
}

impl Default for Wind {
    fn default() -> Self {
        Wind::None
    }
}

bitflags! {
    #[derive(Default)]
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
        count: Option<usize>,
    ) -> Result<Self, insim_core::DecodableError>
    where
        Self: Default,
    {
        let data = Self::default();
        Ok(Self::from_bits_truncate(u16::decode(buf, count)?))
    }
}

impl Encodable for HostFacts {
    fn encode(&self, buf: &mut bytes::BytesMut) -> Result<(), insim_core::EncodableError>
    where
        Self: Sized,
    {
        self.bits().encode(buf)?;
        Ok(())
    }
}

#[derive(Debug, InsimEncode, InsimDecode, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Race Start
pub struct Rst {
    #[insim(pad_bytes_after = "1")]
    pub reqi: RequestId,

    pub racelaps: u8,

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
