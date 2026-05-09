//! Gear

#[derive(Debug, Copy, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
/// Gear
pub enum Gear {
    /// Reverse
    Reverse,
    /// Neutral
    #[default]
    Neutral,
    /// Gear
    Gear(u8),
}

impl crate::Decode for Gear {
    fn decode(ctx: &mut crate::DecodeContext) -> Result<Self, crate::DecodeError> {
        let discrim = ctx.decode::<u8>("discrim")?;
        match discrim {
            0 => Ok(Self::Reverse),
            1 => Ok(Self::Neutral),
            _ => Ok(Self::Gear(discrim - 1)),
        }
    }
}

impl crate::Encode for Gear {
    fn encode(&self, ctx: &mut crate::EncodeContext) -> Result<(), crate::EncodeError> {
        let val: u8 = match self {
            Self::Reverse => 0,
            Self::Neutral => 1,
            Self::Gear(g) => g.saturating_add(1),
        };

        ctx.encode("val", &val)
    }
}
