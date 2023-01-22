use insim_core::prelude::*;

#[cfg(feature = "serde")]
use serde::Serialize;

use bitflags::bitflags;

use crate::protocol::identifiers::RequestId;
use crate::track::Track;

#[derive(Debug, InsimEncode, InsimDecode, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[repr(u8)]
pub enum Wind {
    #[insim(id = "0")]
    None,
    #[insim(id = "1")]
    Weak,
    #[insim(id = "2")]
    Strong,
}

impl Default for Wind {
    fn default() -> Self {
        Wind::None
    }
}

bitflags! {
    #[cfg_attr(feature = "serde", derive(Serialize))]
    pub struct HostFacts: u16 {
        CAN_VOTE => (1 << 0),
        CAN_SELECT => (1 << 1),
        MID_RACE_JOIN => (1 << 2),
        MUST_PIT => (1 << 3),
        CAN_RESET => (1 << 4),
        FORCE_DRIVER_VIEW => (1 << 5),
        CRUISE => (1 << 6),
    }
}

#[derive(Debug, InsimEncode, InsimDecode, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Race Start
pub struct Rst {
    #[insim(pad_bytes_after = "1")]
    pub reqi: RequestId,

    pub racelaps: u8,

    pub qualmins: u8,

    pub nump: u8,

    pub timing: u8,

    pub track: Track,

    pub weather: u8,

    pub wind: Wind,

    pub flags: HostFacts,

    pub numnodes: u16,

    pub finish: u16,

    pub split1: u16,

    pub split2: u16,

    pub split3: u16,
}
