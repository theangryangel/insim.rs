use std::{
    fmt,
    ops::{Deref, DerefMut},
};

use bytes::{Buf, BufMut, Bytes, BytesMut};
use insim_core::{Decode, Encode};

/// Unique Connection Identifier, commonly referred to as UCID in Insim.txt
#[derive(Debug, Ord, PartialOrd, PartialEq, Eq, Hash, Clone, Copy, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ConnectionId(pub u8);

impl ConnectionId {
    #[allow(dead_code)]
    /// Shortcut for local or server
    const LOCAL: ConnectionId = ConnectionId(0);

    #[allow(dead_code)]
    /// Shortcut for commonly used "all" connection id
    const ALL: ConnectionId = ConnectionId(255);
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

impl From<u8> for ConnectionId {
    fn from(value: u8) -> Self {
        Self(value)
    }
}

impl Decode for ConnectionId {
    fn decode(buf: &mut Bytes) -> Result<Self, insim_core::DecodeError> {
        Ok(ConnectionId(buf.get_u8()))
    }
}

impl Encode for ConnectionId {
    fn encode(&self, buf: &mut BytesMut) -> Result<(), insim_core::EncodeError> {
        buf.put_u8(self.0);

        Ok(())
    }
}
