use std::time::Duration;

use insim_core::{identifiers::RequestId, prelude::*};

#[cfg(feature = "serde")]
use serde::Serialize;

#[derive(Debug, Default, InsimEncode, InsimDecode, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[repr(u8)]
pub enum RipError {
    #[default]
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

    pub ctime: Duration,
    pub ttime: Duration,

    #[insim(bytes = "64")]
    pub rname: String,
}
