use crate::string::ICodepageString;
use deku::ctx::Size;
use deku::prelude::*;
use serde::Serialize;

/// Enum for the sound field of [Mso].
#[derive(Debug, PartialEq, DekuRead, DekuWrite, Serialize, Clone)]
#[deku(type = "u8", endian = "little")]
pub enum MsoUserType {
    #[deku(id = "0")]
    /// System message.
    System,

    #[deku(id = "1")]
    /// Normal, visible, user message.
    User,

    #[deku(id = "2")]
    /// Was this message received with the prefix character from the [Init](super::Init) message?
    Prefix,

    #[deku(id = "3")]
    // FIXME: Due to be retired in Insim v9
    O,
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Serialize)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
/// System messsages and user messages, variable sized.
pub struct Mso {
    #[deku(pad_bytes_after = "1")]
    pub reqi: u8,

    pub ucid: u8,

    pub plid: u8,

    /// Set if typed by a user
    pub usertype: MsoUserType,

    /// Index of the first character of user entered text, in msg field.
    pub textstart: u8,

    #[deku(reader = "ICodepageString::read(deku::rest, Size::Bytes(deku::rest.len() / 8))")]
    pub msg: ICodepageString,
}
