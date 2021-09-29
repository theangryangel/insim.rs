use crate::packets::{lfs_string_read, lfs_string_write};
use deku::prelude::*;

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
pub struct Init {
    #[deku(bytes = "1", pad_bytes_after = "1")]
    pub reqi: u8,

    #[deku(bytes = "2")]
    pub udpport: u16,

    #[deku(bytes = "2")]
    pub flags: u16,

    #[deku(bytes = "1")]
    pub version: u8,

    #[deku(bytes = "1")]
    pub prefix: u8,

    #[deku(bytes = "2")]
    pub interval: u16,

    #[deku(
        reader = "lfs_string_read(deku::rest, 16)",
        writer = "lfs_string_write(deku::output, password, 16)"
    )]
    pub password: String,

    #[deku(
        reader = "lfs_string_read(deku::rest, 16)",
        writer = "lfs_string_write(deku::output, name, 16)"
    )]
    pub name: String,
}
