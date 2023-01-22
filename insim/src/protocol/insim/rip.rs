use insim_core::prelude::*;

#[cfg(feature = "serde")]
use serde::Serialize;

use crate::{protocol::identifiers::RequestId, string::istring};

#[derive(Debug, InsimEncode, InsimDecode, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[repr(u8)]
pub enum RipError {
    Ok = 0,

    Already = 1,

    Dedicated = 2,

    WrongMode = 3,

    NotReplay = 4,

    Corrupted = 5,

    NotFound = 6,

    Unloadable = 7,

    DestOOB = 8,

    Unknown = 9,

    User = 10,

    OOS = 11,
}

impl Default for RipError {
    fn default() -> Self {
        RipError::Ok
    }
}

#[derive(Debug, InsimEncode, InsimDecode, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Replay Information
pub struct Rip {
    pub reqi: RequestId,
    pub error: RipError,

    pub mpr: u8,
    pub paused: u8,

    #[insim(pad_bytes_after = "1")]
    pub options: u8, // FIXME: implement flags

    pub ctime: u32,
    pub ttime: u32,

    #[insim(
        reader = "istring::read(insim::rest, 64)",
        writer = "istring::write(insim::output, &self.rname, 64)"
    )]
    pub rname: String,
}
