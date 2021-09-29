use crate::packets::{lfs_string_read, lfs_string_write};
use deku::prelude::*;

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "little")]
pub struct HostInfo {
    #[deku(
        reader = "lfs_string_read(deku::rest, 32)",
        writer = "lfs_string_write(deku::output, hname, 32)"
    )]
    pub hname: String,

    #[deku(
        reader = "lfs_string_read(deku::rest, 6)",
        writer = "lfs_string_write(deku::output, track, 6)"
    )]
    pub track: String,

    #[deku(bytes = "1")]
    pub flags: u8,

    #[deku(bytes = "1")]
    pub numconns: u8,
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
pub struct AdminRequest {
    #[deku(bytes = "1", pad_bytes_after = "1")]
    pub reqi: u8,
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
pub struct AdminResponse {
    #[deku(bytes = "1")]
    pub reqi: u8,
    #[deku(bytes = "1")]
    pub admin: u8,
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
pub struct HostListRequest {
    #[deku(bytes = "1", pad_bytes_after = "1")]
    pub reqi: u8,
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
pub struct HostList {
    #[deku(bytes = "1")]
    pub reqi: u8,

    #[deku(bytes = "1")]
    pub numhosts: u8,

    #[deku(count = "numhosts")]
    pub hinfo: Vec<HostInfo>,
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
pub struct HostSelect {
    #[deku(bytes = "1", pad_bytes_after = "1")]
    pub reqi: u8,

    // zero handled by pad_bytes_after
    #[deku(
        reader = "lfs_string_read(deku::rest, 32)",
        writer = "lfs_string_write(deku::output, hname, 32)"
    )]
    pub hname: String,

    #[deku(
        reader = "lfs_string_read(deku::rest, 16)",
        writer = "lfs_string_write(deku::output, admin, 16)"
    )]
    pub admin: String,

    #[deku(
        reader = "lfs_string_read(deku::rest, 16)",
        writer = "lfs_string_write(deku::output, spec, 16)"
    )]
    pub spec: String,
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
pub struct Error {
    #[deku(bytes = "1")]
    pub reqi: u8,

    #[deku(bytes = "1")]
    pub errno: u8,
}
