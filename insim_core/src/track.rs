#[cfg(feature = "serde")]
use serde::Serialize;

use crate::{
    ser::Limit, string::strip_trailing_nul, Decodable, DecodableError, Encodable, EncodableError,
};

/// Handles parsing a Track name.
#[derive(Debug, PartialEq, Eq, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct Track {
    pub inner: [u8; 6],
}

impl Track {
    /// Is this a reversed track?
    pub fn is_reverse(&self) -> bool {
        matches!(strip_trailing_nul(&self.inner).last(), Some(b'R'))
    }

    /// Are we in open world mode?
    pub fn is_open_world(&self) -> bool {
        matches!(strip_trailing_nul(&self.inner).last(), Some(b'X'))
    }
}

impl std::fmt::Display for Track {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let stripped = strip_trailing_nul(&self.inner);
        write!(f, "{}", String::from_utf8_lossy(stripped))
    }
}

impl Encodable for Track {
    fn encode(
        &self,
        buf: &mut bytes::BytesMut,
        limit: Option<Limit>,
    ) -> Result<(), EncodableError> {
        if let Some(limit) = limit {
            return Err(EncodableError::UnexpectedLimit(format!(
                "Track does not support limit: {limit:?}",
            )));
        }

        for i in self.inner.iter() {
            i.encode(buf, None)?;
        }

        Ok(())
    }
}

impl Decodable for Track {
    fn decode(buf: &mut bytes::BytesMut, limit: Option<Limit>) -> Result<Self, DecodableError> {
        if let Some(limit) = limit {
            return Err(DecodableError::UnexpectedLimit(format!(
                "Track does not support limit: {limit:?}",
            )));
        }

        let mut data: Track = Default::default();
        for i in 0..6 {
            data.inner[i] = u8::decode(buf, None)?;
        }

        Ok(data)
    }
}
