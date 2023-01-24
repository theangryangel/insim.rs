use crate::{Decodable, Encodable};

use std::fmt;
use std::ops::{Deref, DerefMut};

#[derive(Debug, Ord, PartialOrd, PartialEq, Eq, Hash, Clone, Copy, Default)]
pub struct RequestId(pub u8);

impl Encodable for RequestId {
    fn encode(&self, buf: &mut bytes::BytesMut) -> Result<(), crate::EncodableError>
    where
        Self: Sized,
    {
        self.0.encode(buf)?;
        Ok(())
    }
}

impl Decodable for RequestId {
    fn decode(
        buf: &mut bytes::BytesMut,
        count: Option<usize>,
    ) -> Result<Self, crate::DecodableError>
    where
        Self: Default,
    {
        Ok(Self(u8::decode(buf, count)?))
    }
}

impl fmt::Display for RequestId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Deref for RequestId {
    type Target = u8;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for RequestId {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
