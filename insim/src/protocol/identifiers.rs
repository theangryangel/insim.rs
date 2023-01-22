use insim_core::prelude::*;

#[cfg(feature = "serde")]
use serde::Serialize;

use std::fmt;
use std::ops::{Deref, DerefMut};

#[derive(
    Debug, Ord, PartialOrd, PartialEq, Eq, Hash, InsimEncode, InsimDecode, Clone, Copy, Default,
)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct ConnectionId(u8);

impl fmt::Display for ConnectionId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(
    Debug, Ord, PartialOrd, PartialEq, Eq, Hash, InsimEncode, InsimDecode, Clone, Copy, Default,
)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct PlayerId(u8);

impl fmt::Display for PlayerId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(
    Debug, Ord, PartialOrd, PartialEq, Eq, Hash, InsimEncode, InsimDecode, Clone, Copy, Default,
)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct RequestId(pub u8);

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
