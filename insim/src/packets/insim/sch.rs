use bitflags::bitflags;
use insim_core::{identifiers::RequestId, prelude::*, ser::Limit, DecodableError, EncodableError};

#[cfg(feature = "serde")]
use serde::Serialize;

bitflags! {
    /// Bitwise flags used within the [Sch] packet
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Default)]
    #[cfg_attr(feature = "serde", derive(Serialize))]
    pub struct SchFlags: u8 {
        /// Shift
        const SHIFT = (1 << 0);

        /// Ctrl
        const CTRL = (1 << 1);
    }
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Send Single Character
pub struct Sch {
    pub reqi: RequestId,

    pub charb: char,
    pub flags: SchFlags,
}

impl Encodable for Sch {
    fn encode(
        &self,
        buf: &mut bytes::BytesMut,
        limit: Option<Limit>,
    ) -> Result<(), EncodableError> {
        if limit.is_some() {
            return Err(EncodableError::UnexpectedLimit(format!(
                "Sch does not support a limit: {limit:?}",
            )));
        }

        self.reqi.encode(buf, None)?;
        buf.put_bytes(0, 1);

        (self.charb as u8).encode(buf, None)?;
        self.flags.bits().encode(buf, None)?;
        buf.put_bytes(0, 2);

        Ok(())
    }
}

impl Decodable for Sch {
    fn decode(buf: &mut bytes::BytesMut, limit: Option<Limit>) -> Result<Self, DecodableError>
    where
        Self: Default,
    {
        if limit.is_some() {
            return Err(DecodableError::UnexpectedLimit(format!(
                "Sch does not support a limit: {:?}",
                limit
            )));
        }

        let mut data = Self {
            reqi: RequestId::decode(buf, None)?,
            ..Default::default()
        };

        buf.advance(1);

        data.charb = u8::decode(buf, None)? as char;

        data.flags = SchFlags::from_bits_truncate(u8::decode(buf, None)?);
        buf.advance(2);

        Ok(data)
    }
}
