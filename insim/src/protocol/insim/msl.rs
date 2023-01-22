use insim_core::prelude::*;

#[cfg(feature = "serde")]
use serde::Serialize;

use crate::{protocol::identifiers::RequestId, string::CodepageString};

/// Enum for the sound field of [Msl].
#[derive(Debug, InsimEncode, InsimDecode, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[repr(u8)]
pub enum MslSoundType {
    Silent = 0,

    Message = 1,

    SystemMessage = 2,

    InvalidKey = 3,

    // This is referred to as "Error" in the Insim documentation, but this is a special word in
    // rust so I'm trying to avoid it.
    Failure = 4,
}

impl Default for MslSoundType {
    fn default() -> Self {
        MslSoundType::Silent
    }
}

#[derive(Debug, InsimEncode, InsimDecode, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Send a message to the local computer only. If you are connected to a server this means the
/// console. If you are connected to a client this means to the local client only.
pub struct Msl {
    pub reqi: RequestId,

    pub sound: MslSoundType,

    #[deku(bytes = "128")]
    pub msg: CodepageString,
}
