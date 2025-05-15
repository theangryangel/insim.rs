//! Gear

#[derive(Debug, Copy, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
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
    fn decode(buf: &mut bytes::Bytes) -> Result<Self, crate::DecodeError> {
        let discrim = u8::decode(buf)?;
        match discrim {
            0 => Ok(Self::Reverse),
            1 => Ok(Self::Neutral),
            _ => Ok(Self::Gear(discrim - 1)),
        }
    }
}

impl crate::Encode for Gear {
    fn encode(&self, buf: &mut bytes::BytesMut) -> Result<(), crate::EncodeError> {
        let val: u8 = match self {
            Self::Reverse => 0,
            Self::Neutral => 1,
            Self::Gear(g) => g.saturating_add(1),
        };

        val.encode(buf)
    }
}
