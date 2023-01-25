use crate::{Decodable, Encodable, ser::Limit};

use std::fmt;
use std::ops::{Deref, DerefMut};

#[cfg(feature = "serde")]
use serde::Serialize;

#[derive(Debug, Ord, PartialOrd, PartialEq, Eq, Hash, Clone, Copy, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct PlayerId(u8);

impl fmt::Display for PlayerId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Encodable for PlayerId {
    fn encode(&self, buf: &mut bytes::BytesMut, limit: Option<Limit>) -> Result<(), crate::EncodableError>
    where
        Self: Sized,
    {
        self.0.encode(buf, limit)?;
        Ok(())
    }
}

impl Decodable for PlayerId {
    fn decode(
        buf: &mut bytes::BytesMut,
        limit: Option<Limit>,
    ) -> Result<Self, crate::DecodableError>
    where
        Self: Default,
    {
        Ok(Self(u8::decode(buf, limit)?))
    }
}

impl Deref for PlayerId {
    type Target = u8;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for PlayerId {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
