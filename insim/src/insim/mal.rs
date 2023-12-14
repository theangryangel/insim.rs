use insim_core::{
    binrw::{self, binrw},
    identifiers::{ConnectionId, RequestId},
};

#[cfg(feature = "serde")]
use serde::Serialize;

use crate::error::Error;

use std::default::Default;

const MAX_MAL_SIZE: usize = 120;

#[binrw]
#[bw(assert(allowed_mods.len() <= MAX_MAL_SIZE))]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Mods Allowed - restrict the mods that can be used
pub struct Mal {
    pub reqi: RequestId,

    /// Number of mods in this packet
    #[bw(calc = allowed_mods.len() as u8)]
    numm: u8,

    pub ucid: ConnectionId,

    #[brw(pad_after = 2)]
    /// Currently unused
    pub flags: u8,

    #[br(count = numm)]
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
        if self.allowed_mods.len() >= MAX_MAL_SIZE {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "MAL packet count exceeds MAX_MAL_SIZE",
            )
            .into());
        }

        self.allowed_mods.push(mod_id);

        Ok(())
    }

    /// Clear any previously allowed mods.
    pub fn clear(&mut self) -> Result<(), Error> {
        self.allowed_mods.clear();
        Ok(())
    }
}
