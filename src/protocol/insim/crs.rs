use deku::prelude::*;
use serde::Serialize;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Serialize)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
/// Car Reset packet indicates a vehicle has been reset or that a vehicle should be reset by the
/// server.
pub struct Crs {
    #[deku(bytes = "1")]
    pub reqi: u8,

    #[deku(bytes = "1")]
    pub plid: u8,
}
