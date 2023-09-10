use std::fmt;
use std::ops::{Deref, DerefMut};

#[cfg(feature = "serde")]
use serde::Serialize;

#[derive(Debug, Ord, PartialOrd, PartialEq, Eq, Hash, Clone, Copy, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct ConnectionIdentifier(pub usize);

impl fmt::Display for ConnectionIdentifier {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Deref for ConnectionIdentifier {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ConnectionIdentifier {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
