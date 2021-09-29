use crate::packets::{lfs_string_read, lfs_string_write};
use deku::prelude::*;

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
pub struct Version {
    #[deku(bytes = "1", pad_bytes_after = "1")]
    reqi: u8,

    #[deku(
        reader = "lfs_string_read(deku::rest, 8)",
        writer = "lfs_string_write(deku::output, version, 8)"
    )]
    version: String,

    #[deku(
        reader = "lfs_string_read(deku::rest, 6)",
        writer = "lfs_string_write(deku::output, product, 6)"
    )]
    product: String,

    #[deku(bytes = "1", pad_bytes_after = "1")]
    insimver: u16,
}
