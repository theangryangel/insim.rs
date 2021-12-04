use crate::packet_flags;
use crate::string::IString;
use deku::prelude::*;
use serde::Serialize;

packet_flags! {
    #[derive(Serialize)]
    pub struct HostInfoFlags: u8 {
        SPECTATE_PASSWORD => (1 << 0),
        LICENSED => (1 << 1),
        S1 => (1 << 2),
        S2 => (1 << 3),
        FIRST => (1 << 6),
        LAST => (1 << 7),
    }
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Serialize)]
#[deku(endian = "little")]
pub struct HostInfo {
    #[deku(bytes = "32")]
    pub hname: IString,

    #[deku(bytes = "6")]
    pub track: IString,

    #[deku(bytes = "1")]
    pub flags: HostInfoFlags,

    #[deku(bytes = "1")]
    pub numconns: u8,
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Serialize)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
pub struct AdminRequest {
    #[deku(bytes = "1", pad_bytes_after = "1")]
    pub reqi: u8,
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Serialize)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
pub struct AdminResponse {
    #[deku(bytes = "1")]
    pub reqi: u8,
    #[deku(bytes = "1")]
    pub admin: u8,
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Default, Serialize)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
pub struct HostListRequest {
    #[deku(bytes = "1", pad_bytes_after = "1")]
    pub reqi: u8,
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Serialize)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
pub struct HostList {
    #[deku(bytes = "1")]
    pub reqi: u8,

    #[deku(bytes = "1")]
    pub numhosts: u8,

    #[deku(count = "numhosts")]
    pub hinfo: Vec<HostInfo>,
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Default, Serialize)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
pub struct HostSelect {
    #[deku(bytes = "1", pad_bytes_after = "1")]
    pub reqi: u8,

    #[deku(bytes = "32")]
    pub hname: IString,

    #[deku(bytes = "16")]
    pub admin: IString,

    #[deku(bytes = "16")]
    pub spec: IString,
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Serialize)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
pub struct Error {
    #[deku(bytes = "1")]
    pub reqi: u8,

    #[deku(bytes = "1")]
    pub errno: u8,
}
