use insim_core::{
    identifiers::{ConnectionId, RequestId},
    prelude::*,
};

#[cfg(feature = "serde")]
use serde::Serialize;

use crate::error::Error;

use std::default::Default;

const MAX_MAL_SIZE: usize = 120;

#[derive(Debug, InsimEncode, InsimDecode, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Mods Allowed - restrict the mods that can be used
pub struct Mal {
    pub reqi: RequestId,
    /// Number of mods in this packet
    numm: u8,
    pub ucid: ConnectionId,

    #[insim(pad_bytes_after = "2")]
    /// Currently unused
    pub flags: u8,

    #[insim(count = "numm")]
    allowed_mods: Vec<u32>,
}

impl Mal {
    /// Return a list of the allowed mods, in "compressed" form.
    pub fn allowed(&self) -> &[u32] {
        &self.allowed_mods
    }

    /// Push a compressed form of a mod onto the list of allowed mods
    /// and update the count.
    pub fn push(&mut self, mod_id: u32) -> Result<(), Error> {
        if (self.numm as usize) >= MAX_MAL_SIZE {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "MAL packet count exceeds MAX_MAL_SIZE",
            )
            .into());
        }

        self.allowed_mods.push(mod_id);
        self.numm = self.allowed_mods.len() as u8;

        Ok(())
    }

    /// Clear any previously allowed mods.
    pub fn clear(&mut self) -> Result<(), Error> {
        self.allowed_mods.clear();
        self.numm = 0;
        Ok(())
    }
}
