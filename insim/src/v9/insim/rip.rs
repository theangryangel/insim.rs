use std::time::Duration;

use insim_core::{identifiers::RequestId, prelude::*, ser::Limit};

#[cfg(feature = "serde")]
use serde::Serialize;

#[derive(Debug, Default, InsimEncode, InsimDecode, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[repr(u8)]
#[non_exhaustive]
pub enum RipError {
    #[default]
    Ok = 0,

    Already = 1,

    Dedicated = 2,

    WrongMode = 3,

    NotReplay = 4,

    Corrupted = 5,

    NotFound = 6,

    Unloadable = 7,

    DestOOB = 8,

    Unknown = 9,

    User = 10,

    OOS = 11,
}

bitflags::bitflags! {
    /// Bitwise flags used within the [Sta] packet
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Default)]
    #[cfg_attr(feature = "serde", derive(Serialize))]
    pub struct RipOptions: u8 {
        /// Replay will loop
        const LOOP = (1 << 0);

        /// Download missing skins
        const SKINS = (1 << 1);

        /// Use full physics
        const FULL_PHYS = (1 << 2);
    }
}

impl Decodable for RipOptions {
    fn decode(
        buf: &mut bytes::BytesMut,
        limit: Option<Limit>,
    ) -> Result<Self, insim_core::DecodableError>
    where
        Self: Default,
    {
        Ok(Self::from_bits_truncate(u8::decode(buf, limit)?))
    }
}

impl Encodable for RipOptions {
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
/// Replay Information
pub struct Rip {
    pub reqi: RequestId,
    pub error: RipError,

    pub mpr: bool,
    pub paused: bool,
    #[insim(pad_bytes_after = "1")]
    pub options: RipOptions,

    pub ctime: Duration,
    pub ttime: Duration,

    #[insim(bytes = "64")]
    pub rname: String,
}
