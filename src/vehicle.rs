//! Utility functions for working with vehicles and fetching vehicle data.

use crate::string::{is_ascii_alphanumeric, strip_trailing_nul};
use deku::prelude::*;
use serde::Serialize;

/// Handles parsing a vehicle name according to the Insim v9 rules.
/// See https://www.lfs.net/forum/thread/95662-New-InSim-packet-size-byte-and-mod-info
#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Serialize, Default)]
pub struct Vehicle {
    pub inner: [u8; 4],
}

impl Vehicle {
    /// Is this a built in vehicle?
    pub fn is_builtin(&self) -> bool {
        is_ascii_alphanumeric(&self.inner[0])
            && is_ascii_alphanumeric(&self.inner[1])
            && is_ascii_alphanumeric(&self.inner[2])
            && self.inner[3] == 0
    }

    /// Is this a mod? This is only applicable for >= Insim v9.
    pub fn is_mod(&self) -> bool {
        !self.is_builtin()
    }

    /// Determine the mod id. This is only applicable for Insim v9.
    pub fn mod_id_as_string(&self) -> Option<String> {
        if self.is_builtin() {
            return None;
        }
        Some(format!("{:06X}", u32::from_le_bytes(self.inner)))
    }

    /// Return the "uncompressed" mod ID. This is only applicable for Insim v9.
    pub fn mod_id_as_u32(&self) -> Option<u32> {
        if self.is_builtin() {
            return None;
        }
        Some(u32::from_le_bytes(self.inner))
    }
}

impl std::fmt::Display for Vehicle {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if self.is_builtin() {
            let stripped = strip_trailing_nul(&self.inner);
            write!(f, "{}", String::from_utf8_lossy(stripped))
        } else {
            write!(f, "{}", self.mod_id_as_string().unwrap())
        }
    }
}
