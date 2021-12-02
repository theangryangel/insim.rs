use deku::prelude::*;
use serde::Serialize;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Serialize, Clone)]
#[deku(type = "u8", endian = "little")]
pub enum CnlReason {
    #[deku(id = "0")]
    Disconnected,

    #[deku(id = "1")]
    Timeout,

    #[deku(id = "2")]
    LostConnection,

    #[deku(id = "3")]
    Kicked,

    #[deku(id = "4")]
    Banned,

    #[deku(id = "5")]
    Security,

    #[deku(id = "6")]
    Cpw,

    #[deku(id = "7")]
    Oos,

    #[deku(id = "8")]
    Joos,

    #[deku(id = "9")]
    Hack,
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Serialize)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
// Connection Leave
pub struct Cnl {
    #[deku(bytes = "1")]
    pub reqi: u8,

    #[deku(bytes = "1")]
    pub ucid: u8,

    pub reason: CnlReason,

    #[deku(bytes = "1", pad_bytes_after = "2")]
    pub total: u8,
}
