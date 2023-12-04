use binrw::binrw;
#[cfg(feature = "serde")]
use serde::Serialize;

use crate::string::strip_trailing_nul;

/// Handles parsing a Track name.
#[binrw]
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
