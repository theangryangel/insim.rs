//! Identifiers

use std::{
    fmt,
    ops::{Deref, DerefMut},
};

pub use ::insim_core as core;
use insim_core::{Decode, DecodeContext, Encode, EncodeContext};

/// Unique Player Identifier, commonly referred to as PLID in Insim.txt
#[derive(Debug, Ord, PartialOrd, PartialEq, Eq, Hash, Clone, Copy, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OutsimId(pub i32);

impl fmt::Display for OutsimId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Deref for OutsimId {
    type Target = i32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for OutsimId {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<i32> for OutsimId {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

impl Decode for OutsimId {
    fn decode(ctx: &mut DecodeContext) -> Result<Self, insim_core::DecodeError> {
        ctx.decode::<i32>("val").map(OutsimId)
    }
}

impl Encode for OutsimId {
    fn encode(&self, ctx: &mut EncodeContext) -> Result<(), insim_core::EncodeError> {
        ctx.encode("val", &self.0)
    }
}
