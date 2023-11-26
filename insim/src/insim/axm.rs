use insim_core::{
    identifiers::{ConnectionId, RequestId},
    prelude::*,
    ser::Limit,
};

#[cfg(feature = "serde")]
use serde::Serialize;

/// Used within the [Axm] packet.
#[derive(Debug, InsimEncode, InsimDecode, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct ObjectInfo {
    pub x: i16,
    pub y: i16,

    pub z: u8,
    pub flags: u8,
    pub index: u8,
    pub heading: u8,
}

/// Actionst hat can be taken as part of [Axm].
#[derive(Debug, Default, InsimEncode, InsimDecode, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[repr(u8)]
pub enum PmoAction {
    #[default]
    LoadingFile = 0,

    AddObjects = 1,

    DelObjects = 2,

    ClearAll = 3,

    TinyAxm = 4,

    TtcSel = 5,

    Selection = 6,

    Position = 7,

    GetZ = 8,
}

bitflags::bitflags! {
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Default)]
    #[cfg_attr(feature = "serde", derive(Serialize))]
    pub struct PmoFlags: u16 {
         const FILE_END = (1 << 0);
         const MOVE_MODIFY = (1 << 1);
         const SELECTION_REAL = (1 << 2);
         const AVOID_CHECK = (1 << 3);
    }
}

impl Encodable for PmoFlags {
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

impl Decodable for PmoFlags {
    fn decode(
        buf: &mut bytes::BytesMut,
        limit: Option<Limit>,
    ) -> Result<Self, insim_core::DecodableError>
    where
        Self: Sized,
    {
        Ok(Self::from_bits_truncate(u16::decode(buf, limit)?))
    }
}

/// AutoX Multiple Objects
#[derive(Debug, InsimEncode, InsimDecode, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct Axm {
    pub reqi: RequestId,
    pub numo: u8,

    pub ucid: ConnectionId,
    pub action: PmoAction,
    #[insim(pad_bytes_after = "1")]
    pub flags: PmoFlags,

    #[insim(count = "numo")]
    pub info: Vec<ObjectInfo>,
}
