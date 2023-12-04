use insim_core::{
    identifiers::{ConnectionId, PlayerId, RequestId},
    vehicle::Vehicle,
    binrw::{self, binrw},
    string::{binrw_write_codepage_string, binrw_parse_codepage_string}
};

#[cfg(feature = "serde")]
use serde::Serialize;

use bitflags::bitflags;

#[binrw]
#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[repr(u8)]
#[brw(repr(u8))]
pub enum TyreCompound {
    R1 = 0,

    R2 = 1,

    R3 = 2,

    R4 = 3,

    RoadSuper = 4,

    RoadNormal = 5,

    Hybrid = 6,

    Knobbly = 7,

    #[default]
    NoChange = 255,
}

bitflags! {
    #[binrw]
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Default)]
    #[cfg_attr(feature = "serde", derive(Serialize))]
    #[br(map = Self::from_bits_truncate)]
    #[bw(map = |&x: &Self| x.bits())]
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


#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Sent when a New Player joins.
// TODO: Implement SetF, Fuel, RW/FWAdj, PType
pub struct Npl {
    pub reqi: RequestId,
    pub plid: PlayerId,

    pub ucid: ConnectionId,
    pub ptype: u8,
    pub flags: PlayerFlags,

    #[bw(write_with = binrw_write_codepage_string::<24, _>)]
    #[br(parse_with = binrw_parse_codepage_string::<24, _>)]
    pub pname: String,

    #[bw(write_with = binrw_write_codepage_string::<8, _>)]
    #[br(parse_with = binrw_parse_codepage_string::<8, _>)]
    pub plate: String,

    pub cname: Vehicle,

    #[bw(write_with = binrw_write_codepage_string::<16, _>)]
    #[br(parse_with = binrw_parse_codepage_string::<16, _>)]
    pub sname: String,
    pub tyres: [TyreCompound; 4],

    pub h_mass: u8,
    pub h_tres: u8,
    pub model: u8,
    pub pass: u8,

    pub rwadj: u8,
    #[brw(pad_after = 2)]
    pub fwadj: u8,

    pub setf: u8,
    pub nump: u8,
    pub config: u8,
    pub fuel: u8,
}
