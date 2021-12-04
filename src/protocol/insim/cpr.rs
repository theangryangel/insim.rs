use crate::string::IString;
use deku::prelude::*;
use serde::Serialize;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Serialize)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
/// Connection Player Renamed
pub struct Cpr {
    #[deku(bytes = "1")]
    pub reqi: u8,

    #[deku(bytes = "24")]
    pub pname: IString,

    #[deku(bytes = "8")]
    pub plate: IString,
}
