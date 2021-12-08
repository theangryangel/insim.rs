use crate::string::ICodepageString;
use deku::prelude::*;
#[cfg(feature = "serde")]
use serde::Serialize;

/// Enum for the result field of [Acr].
#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
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

/// Admin Command Report
#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[deku(ctx = "_endian: deku::ctx::Endian")]
pub struct Acr {
    #[deku(pad_bytes_after = "1")]
    pub reqi: u8,

    pub ucid: u8,

    pub admin: u8,

    #[deku(pad_bytes_after = "1")]
    pub result: AcrResult,

    #[deku(bytes = "64")]
    pub text: ICodepageString,
}
