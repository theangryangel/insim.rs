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
pub enum TinyType {
    #[deku(id = "0")]
    None,

    #[deku(id = "1")]
    Version,

    #[deku(id = "2")]
    Close,

    #[deku(id = "3")]
    Ping,

    #[deku(id = "4")]
    Pong,

    #[deku(id = "5")]
    Vtc,

    #[deku(id = "6")]
    Scp,

    #[deku(id = "7")]
    Sst,

    #[deku(id = "8")]
    Gth,

    #[deku(id = "9")]
    Mpe,

    #[deku(id = "10")]
    Ism,

    #[deku(id = "11")]
    Ren,

    #[deku(id = "12")]
    Clr,

    #[deku(id = "13")]
    Ncn,

    #[deku(id = "14")]
    Npl,

    #[deku(id = "15")]
    Res,

    #[deku(id = "16")]
    Nlp,

    #[deku(id = "17")]
    Mci,

    #[deku(id = "18")]
    Reo,

    #[deku(id = "19")]
    Rst,

    #[deku(id = "20")]
    Axi,

    #[deku(id = "21")]
    Axc,

    #[deku(id = "22")]
    Rip,

    #[deku(id = "23")]
    Nci,

    #[deku(id = "24")]
    Alc,

    #[deku(id = "25")]
    Axm,

    #[deku(id = "26")]
    Slc,
}

impl Default for TinyType {
    fn default() -> Self {
        TinyType::None
    }
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[deku(
    ctx = "endian: deku::ctx::Endian",
    ctx_default = "deku::ctx::Endian::Little",
    endian = "endian"
)]
/// General purpose Tiny packet
pub struct Tiny {
    pub reqi: u8,

    pub subtype: TinyType,
}
