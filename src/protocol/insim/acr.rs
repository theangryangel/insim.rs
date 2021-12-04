use crate::string::IString;
use deku::prelude::*;
use serde::Serialize;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Serialize, Clone)]
#[deku(type = "u8", endian = "little")]
pub enum AcrResult {
    #[deku(id = "0")]
    None,

    #[deku(id = "1")]
    Processed,

    #[deku(id = "2")]
    Rejected,

    #[deku(id = "3")]
    UnknownCommand,
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Serialize)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
/// Admin Command Report
pub struct Acr {
    #[deku(pad_bytes_after = "1")]
    pub reqi: u8,

    pub ucid: u8,

    pub admin: u8,

    #[deku(pad_bytes_after = "1")]
    pub result: AcrResult,

    #[deku(bytes = "64")]
    pub text: IString,
}
