//! Strongly type license data
use crate::{Decode, Encode};

/// LFS content license tier.
///
/// - Determines which tracks and vehicles are available.
/// - See <https://www.lfs.net/contents> for details.
#[non_exhaustive]
#[derive(Default, PartialEq, PartialOrd, Eq, Ord, Hash, Debug, Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
pub enum License {
    #[default]
    /// Demo
    Demo = 0,
    /// S1
    S1 = 1,
    /// S2
    S2 = 2,
    /// S3
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

impl Decode for License {
    fn decode(ctx: &mut crate::DecodeContext) -> Result<Self, crate::DecodeError> {
        match ctx.decode::<u8>("val")? {
            0 => Ok(Self::Demo),
            1 => Ok(Self::S1),
            2 => Ok(Self::S2),
            3 => Ok(Self::S3),
            other => Err(crate::DecodeErrorKind::NoVariantMatch {
                found: other as u64,
            }
            .into()),
        }
    }
}
impl Encode for License {
    fn encode(&self, ctx: &mut crate::EncodeContext) -> Result<(), crate::EncodeError> {
        ctx.encode("val", &(*self as u8))
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
