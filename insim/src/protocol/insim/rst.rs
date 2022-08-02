use crate::packet_flags;
use crate::track::Track;
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
pub enum Wind {
    #[deku(id = "0")]
    None,
    #[deku(id = "1")]
    Weak,
    #[deku(id = "2")]
    Strong,
}

impl Default for Wind {
    fn default() -> Self {
        Wind::None
    }
}

packet_flags! {
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

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[deku(
    ctx = "endian: deku::ctx::Endian",
    ctx_default = "deku::ctx::Endian::Little",
    endian = "endian"
)]
/// Race Start
pub struct Rst {
    #[deku(pad_bytes_after = "1")]
    pub reqi: u8,

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
