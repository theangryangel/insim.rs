use super::packet::Packet;
use crate::string::InsimString;
use deku::prelude::*;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone)]
#[deku(endian = "little")]
pub struct HostInfo {
    #[deku(bytes = "32")]
    pub hname: InsimString,

    #[deku(bytes = "6")]
    pub track: InsimString,

    #[deku(bytes = "1")]
    pub flags: u8,

    #[deku(bytes = "1")]
    pub numconns: u8,
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
pub struct AdminRequest {
    #[deku(bytes = "1", pad_bytes_after = "1")]
    pub reqi: u8,
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
pub struct AdminResponse {
    #[deku(bytes = "1")]
    pub reqi: u8,
    #[deku(bytes = "1")]
    pub admin: u8,
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Default)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
pub struct HostListRequest {
    #[deku(bytes = "1", pad_bytes_after = "1")]
    pub reqi: u8,
}

// TODO make this a macro
impl From<HostListRequest> for Packet {
    fn from(item: HostListRequest) -> Self {
        Self::RelayHostListRequest(item)
    }
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
pub struct HostList {
    #[deku(bytes = "1")]
    pub reqi: u8,

    #[deku(bytes = "1")]
    pub numhosts: u8,

    #[deku(count = "numhosts")]
    pub hinfo: Vec<HostInfo>,
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Default)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
pub struct HostSelect {
    #[deku(bytes = "1", pad_bytes_after = "1")]
    pub reqi: u8,

    // zero handled by pad_bytes_after
    #[deku(bytes = "32")]
    pub hname: InsimString,

    #[deku(bytes = "16")]
    pub admin: InsimString,

    #[deku(bytes = "16")]
    pub spec: InsimString,
}

impl From<HostSelect> for Packet {
    fn from(item: HostSelect) -> Self {
        Self::RelayHostSelect(item)
    }
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
pub struct Error {
    #[deku(bytes = "1")]
    pub reqi: u8,

    #[deku(bytes = "1")]
    pub errno: u8,
}
