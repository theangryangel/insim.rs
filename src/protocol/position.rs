use deku::prelude::*;
use serde::Serialize;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Serialize)]
pub struct FixedPoint {
    #[deku(bytes = "1")]
    x: u8,

    #[deku(bytes = "1")]
    y: u8,

    #[deku(bytes = "1")]
    z: u8,
}
