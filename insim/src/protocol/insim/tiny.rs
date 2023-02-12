use insim_core::{identifiers::RequestId, prelude::*};

#[cfg(feature = "serde")]
use serde::Serialize;

#[derive(Debug, InsimEncode, InsimDecode, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[repr(u8)]
pub enum TinyType {
    None = 0,

    Version = 1,

    Close = 2,

    Ping = 3,

    Pong = 4,

    Vtc = 5,

    Scp = 6,

    Sst = 7,

    Gth = 8,

    Mpe = 9,

    Ism = 10,

    Ren = 11,

    Clr = 12,

    Ncn = 13,

    Npl = 14,

    Res = 15,

    Nlp = 16,

    Mci = 17,

    Reo = 18,

    Rst = 19,

    Axi = 20,

    Axc = 21,

    Rip = 22,

    Nci = 23,

    Alc = 24,

    Axm = 25,

    Slc = 26,
}

impl Default for TinyType {
    fn default() -> Self {
        TinyType::None
    }
}

#[derive(Debug, InsimEncode, InsimDecode, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// General purpose Tiny packet
pub struct Tiny {
    pub reqi: RequestId,

    pub subtype: TinyType,
}
