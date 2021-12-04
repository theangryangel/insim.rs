use crate::packet_flags;
use crate::string::IString;
use deku::prelude::*;
use serde::Serialize;

packet_flags! {
    #[derive(Serialize)]
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

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Serialize)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
/// New Player
pub struct Npl {
    #[deku(bytes = "1")]
    pub reqi: u8,

    #[deku(bytes = "1")]
    pub plid: u8,

    #[deku(bytes = "1")]
    pub ucid: u8,

    #[deku(bytes = "1")]
    pub ptype: u8,

    #[deku(bytes = "2")]
    pub flags: PlayerFlags,

    #[deku(bytes = "24")]
    pub pname: IString,

    #[deku(bytes = "8")]
    pub plate: IString,

    #[deku(bytes = "4")]
    pub cname: IString,

    #[deku(bytes = "16")]
    pub sname: IString,

    #[deku(bytes = "1", count = "4")]
    pub tyres: Vec<u8>,

    #[deku(bytes = "1")]
    pub h_mass: u8,

    #[deku(bytes = "1")]
    pub h_tres: u8,

    #[deku(bytes = "1")]
    pub model: u8,

    #[deku(bytes = "1")]
    pub pass: u8,

    #[deku(bytes = "1")]
    pub rwadj: u8,

    #[deku(bytes = "1", pad_bytes_after = "2")]
    pub fwadj: u8,

    #[deku(bytes = "1")]
    pub setf: u8,

    #[deku(bytes = "1")]
    pub nump: u8,

    #[deku(bytes = "1")]
    pub config: u8,

    #[deku(bytes = "1")]
    pub fuel: u8,
}
