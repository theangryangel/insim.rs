use crate::{Decodable, DecodableError, Encodable};

#[cfg(feature = "serde")]
use serde::Serialize;

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[repr(u8)]
pub enum Wind {
    None = 0,
    Weak = 1,
    Strong = 2,
}

impl Default for Wind {
    fn default() -> Self {
        Wind::None
    }
}

impl Encodable for Wind {
    fn encode(
        &self,
        buf: &mut bytes::BytesMut,
        limit: Option<crate::ser::Limit>,
    ) -> Result<(), crate::EncodableError>
    where
        Self: Sized,
    {
        let repr: u8 = match self {
            Wind::None => 0,
            Wind::Weak => 1,
            Wind::Strong => 2,
        };

        repr.encode(buf, limit)
    }
}

impl Decodable for Wind {
    fn decode(
        buf: &mut bytes::BytesMut,
        limit: Option<crate::ser::Limit>,
    ) -> Result<Self, crate::DecodableError>
    where
        Self: Default,
    {
        match u8::decode(buf, limit)? {
            0 => Ok(Wind::None),
            1 => Ok(Wind::Weak),
            2 => Ok(Wind::Strong),
            unknown => Err(DecodableError::UnmatchedDiscrimnant(format!(
                "Could not match {unknown} as a wind type"
            ))),
        }
    }
}
