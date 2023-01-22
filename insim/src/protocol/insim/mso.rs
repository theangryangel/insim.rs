use insim_core::prelude::*;

#[cfg(feature = "serde")]
use serde::Serialize;

use crate::protocol::identifiers::{ConnectionId, PlayerId, RequestId};
use crate::string::CodepageString;

/// Enum for the sound field of [Mso].
#[derive(Debug, InsimEncode, InsimDecode, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[repr(u8)]
pub enum MsoUserType {
    /// System message.
    System = 0,

    /// Normal, visible, user message.
    User = 1,

    /// Was this message received with the prefix character from the [Init](super::Init) message?
    Prefix = 2,

    // FIXME: Due to be retired in Insim v9
    O = 3,
}

impl Default for MsoUserType {
    fn default() -> Self {
        MsoUserType::System
    }
}

#[derive(Debug, InsimEncode, InsimDecode, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// System messsages and user messages, variable sized.
pub struct Mso {
    #[deku(pad_bytes_after = "1")]
    pub reqi: RequestId,

    pub ucid: ConnectionId,

    pub plid: PlayerId,

    /// Set if typed by a user
    pub usertype: MsoUserType,

    /// Index of the first character of user entered text, in msg field.
    pub textstart: u8,

    #[deku(reader = "CodepageString::read(deku::rest, Size::Bytes(deku::rest.len() / 8))")]
    pub msg: CodepageString,
}
