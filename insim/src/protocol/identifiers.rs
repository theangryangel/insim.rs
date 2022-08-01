use deku::prelude::*;
#[cfg(feature = "serde")]
use serde::Serialize;
use std::fmt;

#[derive(Debug, Eq, PartialEq, Hash, DekuRead, DekuWrite, Clone, Copy, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[deku(endian = "little")]
pub struct ConnectionId(u8);

impl fmt::Display for ConnectionId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Eq, PartialEq, Hash, DekuRead, DekuWrite, Clone, Copy, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[deku(endian = "little")]
pub struct PlayerId(u8);

impl fmt::Display for PlayerId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
