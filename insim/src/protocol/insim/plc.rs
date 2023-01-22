use insim_core::prelude::*;

#[cfg(feature = "serde")]
use serde::Serialize;

use bitflags::bitflags;

use crate::{
    protocol::identifiers::{ConnectionId, RequestId},
};

bitflags! {
    #[cfg_attr(feature = "serde", derive(Serialize))]
    pub struct PlcAllowedCars: u32 {
        XF_GTI => (1 << 1),
        XR_GT => (1 << 2),
        XR_GT_TURBO => (1 << 3),
        RB4 => (1 << 4),
        FXO_TURBO => (1 << 5),
        LX4 => (1 << 6),
        LX6 => (1 << 7),
        MRT5 => (1 << 8),
        UF_1000 => (1 << 9),
        RACEABOUT => (1 << 10),
        FZ50 => (1 << 11),
        FORMULA_XR => (1 << 12),
        XF_GTR => (1 << 13),
        UF_GTR => (1 << 14),
        FORMULA_V8 => (1 << 15),
        FXO_GTR => (1 << 16),
        XR_GTR => (1 << 17),
        FZ50_GTR => (1 << 18),
        BWM_SAUBER_F1_06 => (1 << 19),
        FORMULA_BMW_FB02 => (1 << 20),
    }
}

#[derive(Debug, InsimEncode, InsimDecode, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Player Cars
pub struct Plc {
    #[insim(pad_bytes_after = "1")]
    pub reqi: RequestId,

    #[insim(pad_bytes_before = "3")]
    pub ucid: ConnectionId,
 
    pub allowed_cars: PlcAllowedCars,
}
