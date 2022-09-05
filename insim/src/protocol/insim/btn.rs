use crate::protocol::identifiers::{ConnectionId, RequestId};
use crate::string::CodepageString;
use deku::prelude::*;
#[cfg(feature = "serde")]
use serde::Serialize;

#[derive(Debug, DekuRead, DekuWrite, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[deku(
    type = "u8",
    ctx = "endian: deku::ctx::Endian",
    ctx_default = "deku::ctx::Endian::Little",
    endian = "endian"
)]
/// Used within [Bfn] to specify the action to take.
pub enum BfnType {
    #[deku(id = "0")]
    DeleteButton,

    #[deku(id = "1")]
    Clear,

    #[deku(id = "2")]
    UserClear,

    #[deku(id = "3")]
    ButtonsRequested,
}

impl Default for BfnType {
    fn default() -> Self {
        BfnType::DeleteButton
    }
}

#[derive(Debug, DekuRead, DekuWrite, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[deku(
    ctx = "endian: deku::ctx::Endian",
    ctx_default = "deku::ctx::Endian::Little",
    endian = "endian"
)]
/// Button Function
pub struct Bfn {
    pub reqi: RequestId,
    pub subt: BfnType,
    pub ucid: ConnectionId,
    pub clickid: u8,
    pub clickmax: u8,
    pub inst: u8,
}

#[derive(Debug, DekuRead, DekuWrite, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[deku(
    ctx = "endian: deku::ctx::Endian",
    ctx_default = "deku::ctx::Endian::Little",
    endian = "endian"
)]
/// Button
pub struct Btn {
    pub reqi: RequestId,
    pub ucid: ConnectionId,
    pub clickid: u8,
    pub inst: u8,
    pub bstyle: u8, // FIXME: implement bit flags
    pub typein: u8,

    pub left: u8,
    pub top: u8,
    pub width: u8,
    pub height: u8,

    #[deku(bytes = "240")]
    pub text: CodepageString, // FIXME: this should be upto 240 characters and always a multiple of 4
}

#[derive(Debug, DekuRead, DekuWrite, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[deku(
    ctx = "endian: deku::ctx::Endian",
    ctx_default = "deku::ctx::Endian::Little",
    endian = "endian"
)]
/// Button Click - Sent back when a user clicks a button
pub struct Btc {
    pub reqi: RequestId,
    pub ucid: ConnectionId,
    pub clickid: u8,
    pub inst: u8,
    #[deku(pad_bytes_after = "1")]
    pub cflags: u8,
}

#[derive(Debug, DekuRead, DekuWrite, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[deku(
    ctx = "endian: deku::ctx::Endian",
    ctx_default = "deku::ctx::Endian::Little",
    endian = "endian"
)]
/// Button Type - Sent back when a user types into a text entry "button"
pub struct Btt {
    pub reqi: RequestId,
    pub ucid: ConnectionId,
    pub clickid: u8,
    pub inst: u8,

    #[deku(pad_bytes_after = "1")]
    pub typein: u8,

    #[deku(bytes = "96")]
    pub text: CodepageString,
}
