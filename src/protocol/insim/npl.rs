use crate::packet_flags;
use crate::string::{istring, CodepageString};
use crate::vehicle::Vehicle;
use deku::prelude::*;
#[cfg(feature = "serde")]
use serde::Serialize;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[deku(type = "u8", endian = "little")]
pub enum TyreCompound {
    #[deku(id = "0")]
    R1,
    #[deku(id = "1")]
    R2,
    #[deku(id = "2")]
    R3,
    #[deku(id = "3")]
    R4,
    #[deku(id = "4")]
    RoadSuper,
    #[deku(id = "5")]
    RoadNormal,
    #[deku(id = "6")]
    Hybrid,
    #[deku(id = "7")]
    Knobbly,
}

packet_flags! {
    #[cfg_attr(feature = "serde", derive(Serialize))]
    pub struct PlayerFlags: u16 {
        SWAPSIDE => (1 << 0),
        RESERVED_2 => (1 << 1),
        RESERVED_4 => (1 << 2),
        AUTOGEARS => (1 << 3),
        SHIFTER => (1 << 4),
        RESERVED_32 => (1 << 5),
        HELP_B => (1 << 6),
        AXIS_CLUTCH => (1 << 7),
        INPITS => (1 << 8),
        AUTOCLUTCH => (1 << 9),
        MOUSE => (1 << 10),
        KB_NO_HELP => (1 << 11),
        KB_STABILISED => (1 << 12),
        CUSTOM_VIEW => (1 << 13),
    }
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[deku(ctx = "_endian: deku::ctx::Endian")]
/// Sent when a New Player joins.
pub struct Npl {
    pub reqi: u8,

    pub plid: u8,

    pub ucid: u8,

    pub ptype: u8,

    pub flags: PlayerFlags,

    #[deku(bytes = "24")]
    pub pname: CodepageString,

    #[deku(bytes = "8")]
    pub plate: CodepageString,

    pub cname: Vehicle,

    #[deku(
        reader = "istring::read(deku::rest, 16)",
        writer = "istring::write(deku::output, &self.sname, 16)"
    )]
    pub sname: String,

    #[deku(count = "4")]
    pub tyres: Vec<TyreCompound>,

    pub h_mass: u8,

    pub h_tres: u8,

    pub model: u8,

    pub pass: u8,

    pub rwadj: u8,

    #[deku(pad_bytes_after = "2")]
    pub fwadj: u8,

    pub setf: u8,

    pub nump: u8,

    pub config: u8,

    pub fuel: u8,
}
