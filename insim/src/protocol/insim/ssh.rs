use crate::string::istring;
use deku::prelude::*;
#[cfg(feature = "serde")]
use serde::Serialize;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[deku(
    type = "u8",
    ctx = "endian: deku::ctx::Endian",
    ctx_default = "deku::ctx::Endian::Little",
    endian = "endian"
)]
pub enum SshError {
    #[deku(id = "0")]
    Ok,

    #[deku(id = "1")]
    Dedicated,

    #[deku(id = "2")]
    Corrupted,

    #[deku(id = "2")]
    NoSave,
}

impl Default for SshError {
    fn default() -> Self {
        SshError::Ok
    }
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[deku(
    ctx = "endian: deku::ctx::Endian",
    ctx_default = "deku::ctx::Endian::Little",
    endian = "endian"
)]
/// Send Screenshot
pub struct Ssh {
    pub reqi: u8,

    #[deku(pad_bytes_after = "4")]
    pub error: u8,

    #[deku(
        reader = "istring::read(deku::rest, 32)",
        writer = "istring::write(deku::output, &self.lname, 32)"
    )]
    pub lname: String,
}
