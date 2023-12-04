use insim_core::{
    binrw::{self, binrw},
    identifiers::{ConnectionId, RequestId},
};

#[cfg(feature = "serde")]
use serde::Serialize;

use bitflags::bitflags;

bitflags! {
    #[binrw]
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Default)]
    #[cfg_attr(feature = "serde", derive(Serialize))]
    #[br(map = Self::from_bits_truncate)]
    #[bw(map = |&x: &Self| x.bits())]
    pub struct PlcAllowedCars: u32 {
         const XF_GTI = (1 << 1);
         const XR_GT = (1 << 2);
         const XR_GT_TURBO = (1 << 3);
         const RB4 = (1 << 4);
         const FXO_TURBO = (1 << 5);
         const LX4 = (1 << 6);
         const LX6 = (1 << 7);
         const MRT5 = (1 << 8);
         const UF_1000 = (1 << 9);
         const RACEABOUT = (1 << 10);
         const FZ50 = (1 << 11);
         const FORMULA_XR = (1 << 12);
         const XF_GTR = (1 << 13);
         const UF_GTR = (1 << 14);
         const FORMULA_V8 = (1 << 15);
         const FXO_GTR = (1 << 16);
         const XR_GTR = (1 << 17);
         const FZ50_GTR = (1 << 18);
         const BWM_SAUBER_F1_06 = (1 << 19);
         const FORMULA_BMW_FB02 = (1 << 20);
    }
}

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Player Cars
pub struct Plc {
    #[brw(pad_after = 1)]
    pub reqi: RequestId,

    #[brw(pad_before = 3)]
    pub ucid: ConnectionId,

    pub allowed_cars: PlcAllowedCars,
}
