use insim_core::{
    identifiers::{ConnectionId, RequestId},
    prelude::*,
    string::CodepageString,
};

#[cfg(feature = "serde")]
use serde::Serialize;

#[derive(Debug, InsimEncode, InsimDecode, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[repr(u8)]
/// Used within [Bfn] to specify the action to take.
pub enum BfnType {
    DeleteButton = 0,

    Clear = 1,

    UserClear = 2,

    ButtonsRequested = 3,
}

impl Default for BfnType {
    fn default() -> Self {
        BfnType::DeleteButton
    }
}

#[derive(Debug, InsimEncode, InsimDecode, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Button Function
pub struct Bfn {
    pub reqi: RequestId,
    pub subt: BfnType,
    pub ucid: ConnectionId,
    pub clickid: u8,
    pub clickmax: u8,
    pub inst: u8,
}

#[derive(Debug, InsimEncode, InsimDecode, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
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

    #[insim(bytes = "240")]
    pub text: CodepageString, // FIXME: this should be upto 240 characters and always a multiple of 4
}

#[derive(Debug, InsimEncode, InsimDecode, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Button Click - Sent back when a user clicks a button
pub struct Btc {
    pub reqi: RequestId,
    pub ucid: ConnectionId,
    pub clickid: u8,
    pub inst: u8,
    #[insim(pad_bytes_after = "1")]
    pub cflags: u8,
}

#[derive(Debug, InsimEncode, InsimDecode, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Button Type - Sent back when a user types into a text entry "button"
pub struct Btt {
    pub reqi: RequestId,
    pub ucid: ConnectionId,
    pub clickid: u8,
    pub inst: u8,

    #[insim(pad_bytes_after = "1")]
    pub typein: u8,

    #[insim(bytes = "96")]
    pub text: CodepageString,
}
