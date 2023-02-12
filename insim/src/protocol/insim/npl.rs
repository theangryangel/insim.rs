use bytes::BytesMut;
use insim_core::{
    identifiers::{ConnectionId, PlayerId, RequestId},
    prelude::*,
    ser::Limit,
    DecodableError, EncodableError,
};

#[cfg(feature = "serde")]
use serde::Serialize;

use crate::vehicle::Vehicle;
use bitflags::bitflags;

#[derive(Debug, InsimEncode, InsimDecode, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[repr(u8)]
pub enum TyreCompound {
    R1 = 0,

    R2 = 1,

    R3 = 2,

    R4 = 3,

    RoadSuper = 4,

    RoadNormal = 5,

    Hybrid = 6,

    Knobbly = 7,

    NoChange = 255,
}

impl Default for TyreCompound {
    fn default() -> Self {
        TyreCompound::NoChange
    }
}

#[derive(Debug, Clone, Default)]
// we need to wrap [TyreCompound; 4] in a new type because arrays are always considered "foreign", and the trait Decodable isn't defined within this crate.
// FIXME: add some extra methods for convenience
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct TyreCompoundList([TyreCompound; 4]);

impl Decodable for TyreCompoundList {
    fn decode(buf: &mut BytesMut, _limit: Option<Limit>) -> Result<Self, DecodableError> {
        let mut data: TyreCompoundList = Default::default();
        for i in 0..4 {
            data.0[i] = TyreCompound::decode(buf, None)?;
        }

        Ok(data)
    }
}

impl Encodable for TyreCompoundList {
    fn encode(&self, buf: &mut BytesMut, _limit: Option<Limit>) -> Result<(), EncodableError>
    where
        Self: Sized,
    {
        for i in self.0.iter() {
            i.encode(buf, None)?;
        }

        Ok(())
    }
}

bitflags! {
    #[derive(Default)]
    #[cfg_attr(feature = "serde", derive(Serialize))]
    pub struct PlayerFlags: u16 {
         const SWAPSIDE = (1 << 0);
         const RESERVED_2 = (1 << 1);
         const RESERVED_4 = (1 << 2);
         const AUTOGEARS = (1 << 3);
         const SHIFTER = (1 << 4);
         const RESERVED_32 = (1 << 5);
         const HELP_B = (1 << 6);
         const AXIS_CLUTCH = (1 << 7);
         const INPITS = (1 << 8);
         const AUTOCLUTCH = (1 << 9);
         const MOUSE = (1 << 10);
         const KB_NO_HELP = (1 << 11);
         const KB_STABILISED = (1 << 12);
         const CUSTOM_VIEW = (1 << 13);
    }
}

impl Encodable for PlayerFlags {
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

impl Decodable for PlayerFlags {
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

#[derive(Debug, InsimEncode, InsimDecode, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Sent when a New Player joins.
pub struct Npl {
    pub reqi: RequestId,

    pub plid: PlayerId,

    pub ucid: ConnectionId,

    pub ptype: u8,

    pub flags: PlayerFlags,

    #[insim(bytes = "24")]
    pub pname: String,

    #[insim(bytes = "8")]
    pub plate: String,

    pub cname: Vehicle,

    #[insim(bytes = "16")]
    pub sname: String,

    pub tyres: TyreCompoundList,

    pub h_mass: u8,

    pub h_tres: u8,

    pub model: u8,

    pub pass: u8,

    pub rwadj: u8,

    #[insim(pad_bytes_after = "2")]
    pub fwadj: u8,

    pub setf: u8,

    pub nump: u8,

    pub config: u8,

    pub fuel: u8,
}
