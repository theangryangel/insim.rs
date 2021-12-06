use super::VehicleName;
use deku::prelude::*;
use serde::Serialize;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Serialize)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
/// User Selected Car
pub struct Slc {
    pub reqi: u8,

    pub ucid: u8,

    pub cname: VehicleName,
}
