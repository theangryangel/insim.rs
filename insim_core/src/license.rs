//! Strongly type license data
use crate::FromToBytes;

/// Describes the various LFS "license" levels. Each "license" provides access to different
/// levels of content.
/// See <https://www.lfs.net/contents>
#[non_exhaustive]
#[derive(Default, PartialEq, PartialOrd, Eq, Debug, Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[repr(u8)]
pub enum License {
    #[default]
    /// Demo
    Demo = 0,
    /// S1
    S1 = 1,
    /// S2
    S2 = 2,
    /// S2
    S3 = 3,
}

impl std::fmt::Display for License {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            License::Demo => write!(f, "Demo"),
            License::S1 => write!(f, "S1"),
            License::S2 => write!(f, "S2"),
            License::S3 => write!(f, "S3"),
        }
    }
}

impl FromToBytes for License {
    fn from_bytes(buf: &mut bytes::Bytes) -> Result<Self, crate::Error> {
        match u8::from_bytes(buf)? {
            0 => Ok(Self::Demo),
            1 => Ok(Self::S1),
            2 => Ok(Self::S2),
            3 => Ok(Self::S3),
            other => Err(crate::Error::NoVariantMatch{pos: 0, found: other as u64})
        }
    }

    fn to_bytes(&self, buf: &mut bytes::BytesMut) -> Result<(), crate::Error> {
        (*self as u8).to_bytes(buf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_license_order() {
        assert!(License::S3 > License::S2);
        assert!(License::S3 > License::S1);
        assert!(License::S3 > License::Demo);

        assert!(License::S2 > License::S1);
        assert!(License::S2 > License::Demo);

        assert!(License::S1 > License::Demo);
    }
}
