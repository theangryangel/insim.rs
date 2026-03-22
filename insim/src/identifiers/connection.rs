use std::{
    fmt,
    ops::{Deref, DerefMut},
};

use bytes::{Buf, BufMut};
use insim_core::{Decode, DecodeContext, Encode, EncodeContext};

/// Unique Connection Identifier, commonly referred to as UCID in Insim.txt
#[derive(Debug, Ord, PartialOrd, PartialEq, Eq, Hash, Clone, Copy, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ConnectionId(pub u8);

impl ConnectionId {
    /// Shortcut for local or server
    pub const LOCAL: ConnectionId = ConnectionId(0);

    /// Shortcut for commonly used "all" connection id
    pub const ALL: ConnectionId = ConnectionId(255);

    /// Is this a "local" connection
    pub fn local(&self) -> bool {
        matches!(self, &Self::LOCAL)
    }

    /// Is this referencing "all" connections
    pub fn all(&self) -> bool {
        matches!(self, &Self::ALL)
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

impl From<u8> for ConnectionId {
    fn from(value: u8) -> Self {
        Self(value)
    }
}

impl Decode for ConnectionId {
    const PRIMITIVE: bool = true;
    fn decode(ctx: &mut DecodeContext) -> Result<Self, insim_core::DecodeError> {
        Ok(ConnectionId(ctx.buf.get_u8()))
    }
}

impl Encode for ConnectionId {
    const PRIMITIVE: bool = true;
    fn encode(&self, ctx: &mut EncodeContext) -> Result<(), insim_core::EncodeError> {
        ctx.buf.put_u8(self.0);
        Ok(())
    }
}
