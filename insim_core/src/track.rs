use crate::string::{binrw_parse_codepage_string, binrw_write_codepage_string};
use binrw::binrw;

/// Handles parsing a Track name.
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Track {
    #[br(parse_with = binrw_parse_codepage_string::<6, _>, args(true))]
    #[bw(write_with = binrw_write_codepage_string::<6, _>, args(true, 0))]
    pub inner: String,
}

impl Track {
    /// Is this a reversed track?
    pub fn is_reverse(&self) -> bool {
        matches!(&self.inner.chars().last(), Some('R'))
    }

    /// Are we in open world mode?
    pub fn is_open_world(&self) -> bool {
        matches!(&self.inner.chars().last(), Some('X'))
    }
}

impl std::fmt::Display for Track {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_open_world() {
        let track = Track {
            inner: "BL1X".into(),
        };

        assert!(track.is_open_world());
        assert!(!track.is_reverse());
    }

    #[test]
    fn test_reverse() {
        let track = Track {
            inner: "BL1R".into(),
        };

        assert!(!track.is_open_world());
        assert!(track.is_reverse());
    }
}
