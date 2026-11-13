//! Strongly typed wind strength
use crate::{Decode, Encode};

#[derive(Debug, Default, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
/// Wind strength levels.
///
/// - Discrete values (not a continuous scale).
/// - Reported in session state packets.
pub enum Wind {
    #[default]
    /// No wind
    None = 0,
    /// Weak wind
    Weak = 1,
    /// Strong wind
    Strong = 2,
}

impl Decode for Wind {
    fn decode(ctx: &mut crate::DecodeContext) -> Result<Self, crate::DecodeError> {
        match ctx.decode::<u8>("val")? {
            0 => Ok(Wind::None),
            1 => Ok(Self::Weak),
            2 => Ok(Self::Strong),
            other => Err(crate::DecodeErrorKind::NoVariantMatch {
                found: other as u64,
            }
            .into()),
        }
    }
}

impl Encode for Wind {
    fn encode(&self, ctx: &mut crate::EncodeContext) -> Result<(), crate::EncodeError> {
        ctx.encode("val", &(*self as u8))
    }
}
