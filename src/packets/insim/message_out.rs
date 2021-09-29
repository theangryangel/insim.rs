use crate::packets::{lfs_string_read, lfs_string_write};
use deku::prelude::*;

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
pub struct MessageOut {
    #[deku(bytes = "1", pad_bytes_after = "1")]
    pub reqi: u8,

    #[deku(bytes = "1")]
    pub ucid: u8,

    #[deku(bytes = "1")]
    pub plid: u8,

    #[deku(bytes = "1")]
    pub usertype: u8,

    #[deku(bytes = "1")]
    pub textstart: u8,

    #[deku(
        reader = "lfs_string_read(deku::rest, deku::rest.len() / 8)",
        writer = "lfs_string_write(deku::output, msg, 128)"
    )]
    pub msg: String,
}
