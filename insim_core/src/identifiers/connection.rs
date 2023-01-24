use crate::{Decodable, Encodable};

use std::fmt;
use std::ops::{Deref, DerefMut};

#[derive(Debug, Ord, PartialOrd, PartialEq, Eq, Hash, Clone, Copy, Default)]
pub struct ConnectionId(u8);

impl Encodable for ConnectionId {
    fn encode(&self, buf: &mut bytes::BytesMut) -> Result<(), crate::EncodableError>
    where
        Self: Sized,
    {
        self.0.encode(buf)?;
        Ok(())
    }
}

impl Decodable for ConnectionId {
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

impl fmt::Display for ConnectionId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Deref for ConnectionId {
    type Target = u8;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ConnectionId {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
