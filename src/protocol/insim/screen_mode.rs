use deku::prelude::*;
use serde::Serialize;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Serialize)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
pub struct ScreenMode {
    #[deku(bytes = "1", pad_bytes_after = "1")]
    reqi: u8,

    #[deku(bytes = "1")]
    bit16: i8,

    #[deku(bytes = "1")]
    rr: i8,

    #[deku(bytes = "1")]
    width: i8,

    #[deku(bytes = "1")]
    height: i8,
}
