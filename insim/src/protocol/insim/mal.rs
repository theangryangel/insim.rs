use crate::{error::Error, protocol::identifiers::ConnectionId};
use deku::prelude::*;
#[cfg(feature = "serde")]
use serde::Serialize;
use std::default::Default;

const MAX_MAL_SIZE: usize = 120;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[deku(ctx = "_endian: deku::ctx::Endian")]
/// Mods Allowed - restrict the mods that can be used
pub struct Mal {
    pub reqi: u8,

    count: u8,

    pub ucid: ConnectionId,

    #[deku(pad_bytes_after = "2")]
    /// Currently unused
    pub flags: u8,

    #[deku(count = "count")]
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
        if (self.count as usize) >= MAX_MAL_SIZE {
            return Err(Error::TooLarge);
        }

        self.allowed_mods.push(mod_id);
        self.update()?;

        Ok(())
    }

    /// Clear any previously allowed mods.
    pub fn clear(&mut self) -> Result<(), Error> {
        self.allowed_mods.clear();
        self.update()?;
        Ok(())
    }
}
