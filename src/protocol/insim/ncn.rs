use crate::string::{ICodepageString, IString};
use deku::prelude::*;
#[cfg(feature = "serde")]
use serde::Serialize;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[deku(ctx = "_endian: deku::ctx::Endian")]
/// New Connection
pub struct Ncn {
    pub reqi: u8,

    pub ucid: u8,

    #[deku(bytes = "24")]
    /// Username.
    pub uname: IString,

    #[deku(bytes = "24")]
    /// Playername.
    pub pname: ICodepageString,

    /// 1 if administrative user.
    pub admin: u8,

    /// Total number of connections now this player has joined.
    pub total: u8,

    #[deku(pad_bytes_after = "1")]
    pub flags: u8,
}
