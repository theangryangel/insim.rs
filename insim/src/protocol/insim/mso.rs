use crate::protocol::identifiers::{ConnectionId, PlayerId};
use crate::string::CodepageString;
use deku::ctx::Size;
use deku::prelude::*;
#[cfg(feature = "serde")]
use serde::Serialize;

/// Enum for the sound field of [Mso].
#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[deku(
    type = "u8",
    ctx = "endian: deku::ctx::Endian",
    ctx_default = "deku::ctx::Endian::Little",
    endian = "endian"
)]
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

impl Default for MsoUserType {
    fn default() -> Self {
        MsoUserType::System
    }
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[deku(
    ctx = "endian: deku::ctx::Endian",
    ctx_default = "deku::ctx::Endian::Little",
    endian = "endian"
)]
/// System messsages and user messages, variable sized.
pub struct Mso {
    #[deku(pad_bytes_after = "1")]
    pub reqi: u8,

    pub ucid: ConnectionId,

    pub plid: PlayerId,

    /// Set if typed by a user
    pub usertype: MsoUserType,

    /// Index of the first character of user entered text, in msg field.
    pub textstart: u8,

    #[deku(reader = "CodepageString::read(deku::rest, Size::Bytes(deku::rest.len() / 8))")]
    pub msg: CodepageString,
}
