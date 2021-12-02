use deku::prelude::*;
use serde::Serialize;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Serialize)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
/// Screen Mode (originally IS_MOD)
pub struct Mode {
    #[deku(bytes = "1", pad_bytes_after = "1")]
    pub reqi: u8,

    #[deku(bytes = "1")]
    pub bit16: i8,

    #[deku(bytes = "1")]
    pub rr: i8,

    #[deku(bytes = "1")]
    pub width: i8,

    #[deku(bytes = "1")]
    pub height: i8,
}
