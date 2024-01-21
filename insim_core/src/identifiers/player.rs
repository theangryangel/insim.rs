use binrw::binrw;
use std::fmt;
use std::ops::{Deref, DerefMut};

#[cfg(feature = "serde")]
use serde::Serialize;

/// Unique Player Identifier, commonly referred to as PLID in Insim.txt
#[binrw]
#[derive(Debug, Ord, PartialOrd, PartialEq, Eq, Hash, Clone, Copy, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct PlayerId(pub u8);

impl fmt::Display for PlayerId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
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
