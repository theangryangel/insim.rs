use crate::string::IString;
use deku::prelude::*;
use serde::Serialize;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Serialize, Clone)]
#[deku(type = "u8", endian = "little")]
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

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Serialize)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
/// Button Function
pub struct Bfn {
    pub reqi: u8,
    pub subt: BfnType,
    pub ucid: u8,
    pub clickid: u8,
    pub clickmax: u8,
    pub inst: u8,
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Serialize)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
/// Button
pub struct Btn {
    pub reqi: u8,
    pub ucid: u8,
    pub clickid: u8,
    pub inst: u8,
    pub bstyle: u8, // FIXME: implement bit flags
    pub typein: u8,

    pub left: u8,
    pub top: u8,
    pub width: u8,
    pub height: u8,

    #[deku(bytes = "240")]
    pub text: IString, // FIXME: this should be upto 240 characters and always a multiple of 4
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Serialize)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
/// Button Click - Sent back when a user clicks a button
pub struct Btc {
    pub reqi: u8,
    pub ucid: u8,
    pub clickid: u8,
    pub inst: u8,
    #[deku(pad_bytes_after = "1")]
    pub cflags: u8,
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Serialize)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
/// Button Type - Sent back when a user types into a text entry "button"
pub struct Btt {
    pub reqi: u8,
    pub ucid: u8,
    pub clickid: u8,
    pub inst: u8,

    #[deku(pad_bytes_after = "1")]
    pub typein: u8,

    #[deku(bytes = "96")]
    pub text: IString,
}
