use crate::string::InsimString;
use deku::prelude::*;
use serde::Serialize;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Serialize)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
pub struct Npl {
    #[deku(bytes = "1")]
    reqi: u8,

    #[deku(bytes = "1")]
    plid: u8,

    #[deku(bytes = "1")]
    ptype: u8,

    #[deku(bytes = "2")]
    flags: u16,

    #[deku(bytes = "24")]
    pname: InsimString,

    #[deku(bytes = "6")]
    plate: InsimString,

    #[deku(bytes = "4")]
    cname: InsimString,

    #[deku(bytes = "16")]
    sname: InsimString,

    #[deku(bytes = "4")]
    tyres: [u8; 4],

    #[deku(bytes = "1")]
    h_mass: u8,

    #[deku(bytes = "1")]
    h_tres: u8,

    #[deku(bytes = "1")]
    model: u8,

    #[deku(bytes = "1", pad_bytes_after = "4")]
    pass: u8,

    #[deku(bytes = "1")]
    setf: u8,

    #[deku(bytes = "1", pad_bytes_after = "2")]
    nump: u8,
}
