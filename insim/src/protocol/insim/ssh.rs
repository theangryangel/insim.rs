use insim_core::prelude::*;

#[cfg(feature = "serde")]
use serde::Serialize;

use crate::{protocol::identifiers::RequestId, string::istring};

#[derive(Debug, InsimEncode, InsimDecode, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[repr(u8)]
pub enum SshError {
    Ok = 0,

    Dedicated = 1,

    Corrupted = 2,

    NoSave = 2,
}

impl Default for SshError {
    fn default() -> Self {
        SshError::Ok
    }
}

#[derive(Debug, InsimEncode, InsimDecode, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Send Screenshot
pub struct Ssh {
    pub reqi: RequestId,

    #[deku(pad_bytes_after = "4")]
    pub error: u8,

    #[deku(
        reader = "istring::read(deku::rest, 32)",
        writer = "istring::write(deku::output, &self.lname, 32)"
    )]
    pub lname: String,
}
