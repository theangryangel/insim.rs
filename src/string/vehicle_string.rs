use super::{is_ascii_alphanumeric, strip_trailing_nul};
use deku::prelude::*;
use serde::Serialize;

/// Handles parsing a vehicle name according to the Insim v9 rules.
/// See https://www.lfs.net/forum/thread/95662-New-InSim-packet-size-byte-and-mod-info
#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Serialize, Default)]
pub struct IVehicleString {
    pub inner: [u8; 4],
}

impl IVehicleString {
    /// Is this a built in vehicle?
    pub fn is_builtin(&self) -> bool {
        is_ascii_alphanumeric(&self.inner[0])
            && is_ascii_alphanumeric(&self.inner[1])
            && is_ascii_alphanumeric(&self.inner[2])
            && self.inner[3] == 0
    }

    /// Is this a mod? This is only applicable for >= Insim v9
    pub fn is_mod(&self) -> bool {
        !self.is_builtin()
    }

    /// Determine the mod id. This is only applicable for Insim v9
    pub fn mod_id(&self) -> String {
        let value = u32::from_le_bytes(self.inner);
        format!("{:06X}", value)
    }
}

impl std::fmt::Display for IVehicleString {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if self.is_builtin() {
            let stripped = strip_trailing_nul(&self.inner);
            write!(f, "{}", String::from_utf8_lossy(stripped))
        } else {
            write!(f, "{}", self.mod_id())
        }
    }
}
