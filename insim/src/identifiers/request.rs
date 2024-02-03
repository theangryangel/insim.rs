use std::{
    fmt,
    ops::{Deref, DerefMut},
};

use insim_core::binrw::{self as binrw, binrw};
#[cfg(feature = "serde")]
use serde::Serialize;

/// Request Identifier, commonly referred to as reqi in Insim.txt
#[binrw]
#[derive(Debug, Ord, PartialOrd, PartialEq, Eq, Hash, Clone, Copy, Default)]
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
