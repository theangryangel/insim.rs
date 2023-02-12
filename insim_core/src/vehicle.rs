//! Utility functions for working with vehicles and fetching vehicle data.

use crate::{
    ser::Limit,
    string::{is_ascii_alphanumeric, strip_trailing_nul},
    Decodable, DecodableError, Encodable, EncodableError,
};
use bytes::BytesMut;

#[cfg(feature = "serde")]
use serde::Serialize;

/// Handles parsing a vehicle name according to the Insim v9 rules.
/// See <https://www.lfs.net/forum/thread/95662-New-InSim-packet-size-byte-and-mod-info>
#[derive(Debug, PartialEq, Eq, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
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

impl Encodable for Vehicle {
    fn encode(
        &self,
        buf: &mut bytes::BytesMut,
        limit: Option<Limit>,
    ) -> Result<(), EncodableError> {
        if let Some(limit) = limit {
            return Err(EncodableError::UnexpectedLimit(format!(
                "Vehicle does not support limit: {limit:?}",
            )));
        }

        for i in self.inner.iter() {
            i.encode(buf, None)?;
        }

        Ok(())
    }
}

impl Decodable for Vehicle {
    fn decode(buf: &mut BytesMut, limit: Option<Limit>) -> Result<Self, DecodableError> {
        if let Some(limit) = limit {
            return Err(DecodableError::UnexpectedLimit(format!(
                "Vehicle does not support limit: {limit:?}",
            )));
        }

        let mut data: Self = Default::default();
        for i in 0..4 {
            data.inner[i] = u8::decode(buf, None)?;
        }

        Ok(data)
    }
}
