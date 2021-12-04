use crate::string::IString;
use deku::prelude::*;
use serde::Serialize;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Serialize)]
#[deku(endian = "little", type = "u8")]
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

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Serialize)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
/// Send Screenshot
pub struct Ssh {
    pub reqi: u8,

    #[deku(pad_bytes_after = "4")]
    pub error: u8,

    #[deku(bytes = "32")]
    pub lname: IString,
}
