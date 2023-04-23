use std::time::Duration;

use insim_core::{
    identifiers::{PlayerId, RequestId},
    prelude::*,
};

#[cfg(feature = "serde")]
use serde::Serialize;

use super::{PenaltyInfo, PlayerFlags};

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Default, Clone)]
pub enum Fuel200 {
    Percentage(u8),

    #[default]
    No,
}

impl Decodable for Fuel200 {
    fn decode(
        buf: &mut bytes::BytesMut,
        limit: Option<insim_core::ser::Limit>,
    ) -> Result<Self, insim_core::DecodableError>
    where
        Self: Sized,
    {
        let data = u8::decode(buf, limit)?;

        if data == 255 {
            Ok(Fuel200::No)
        } else {
            Ok(Fuel200::Percentage(data))
        }
    }
}

impl Encodable for Fuel200 {
    fn encode(
        &self,
        buf: &mut bytes::BytesMut,
        limit: Option<insim_core::ser::Limit>,
    ) -> Result<(), insim_core::EncodableError>
    where
        Self: Sized,
    {
        match self {
            Fuel200::Percentage(data) => data.encode(buf, limit),
            Fuel200::No => (255 as u8).encode(buf, limit),
        }
    }
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Default, Clone)]
pub enum Fuel {
    Percentage(u8),

    #[default]
    No,
}

impl Decodable for Fuel {
    fn decode(
        buf: &mut bytes::BytesMut,
        limit: Option<insim_core::ser::Limit>,
    ) -> Result<Self, insim_core::DecodableError>
    where
        Self: Sized,
    {
        let data = u8::decode(buf, limit)?;

        if data == 255 {
            Ok(Fuel::No)
        } else {
            Ok(Fuel::Percentage(data))
        }
    }
}

impl Encodable for Fuel {
    fn encode(
        &self,
        buf: &mut bytes::BytesMut,
        limit: Option<insim_core::ser::Limit>,
    ) -> Result<(), insim_core::EncodableError>
    where
        Self: Sized,
    {
        match self {
            Fuel::Percentage(data) => data.encode(buf, limit),
            Fuel::No => (255 as u8).encode(buf, limit),
        }
    }
}

#[derive(Debug, InsimEncode, InsimDecode, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Lap Time for a given player.
pub struct Lap {
    pub reqi: RequestId,
    pub plid: PlayerId,

    pub ltime: Duration, // lap time (ms)
    pub etime: Duration,

    pub lapsdone: u16,
    #[insim(pad_bytes_after = "1")]
    pub flags: PlayerFlags,

    pub penalty: PenaltyInfo,
    pub numstops: u8,
    pub fuel200: Fuel200,
}
