use crate::string::CodepageString;
use deku::prelude::*;
#[cfg(feature = "serde")]
use serde::Serialize;

/// Enum for the sound field of [Msl].
#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[deku(type = "u8", endian = "little")]
pub enum MslSoundType {
    #[deku(id = "0")]
    Silent,

    #[deku(id = "1")]
    Message,

    #[deku(id = "2")]
    SystemMessage,

    #[deku(id = "3")]
    InvalidKey,

    #[deku(id = "4")]
    // This is referred to as "Error" in the Insim documentation, but this is a special word in
    // rust so I'm trying to avoid it.
    Failure,
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[deku(ctx = "_endian: deku::ctx::Endian")]
/// Send a message to the local computer only. If you are connected to a server this means the
/// console. If you are connected to a client this means to the local client only.
pub struct Msl {
    pub reqi: u8,

    pub sound: MslSoundType,

    #[deku(bytes = "128")]
    pub msg: CodepageString,
}
