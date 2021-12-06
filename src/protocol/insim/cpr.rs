use crate::string::ICodepageString;
use deku::prelude::*;
use serde::Serialize;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Serialize)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
/// Connection Player Renamed indicates that a player has changed their name.
pub struct Cpr {
    #[deku(bytes = "1")]
    pub reqi: u8,

    #[deku(bytes = "24")]
    pub pname: ICodepageString,

    #[deku(bytes = "8")]
    pub plate: ICodepageString,
}
