use insim_core::{identifiers::RequestId, prelude::*};

#[cfg(feature = "serde")]
use serde::Serialize;

#[derive(Debug, InsimEncode, InsimDecode, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[repr(u8)]
pub enum SshError {
    Ok = 0,

    Dedicated = 1,

    Corrupted = 2,

    NoSave = 3,
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

    #[insim(pad_bytes_after = "4")]
    pub error: u8,

    #[insim(bytes = "32")]
    pub lname: String,
}
