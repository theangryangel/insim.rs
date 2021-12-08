//! Utility functions for working with positions.

use deku::prelude::*;
#[cfg(feature = "serde")]
use serde::Serialize;

/// A X, Y, Z position
#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct FixedPoint {
    #[deku(bytes = "4")]
    x: i32,

    #[deku(bytes = "4")]
    y: i32,

    #[deku(bytes = "4")]
    z: i32,
}

impl FixedPoint {
    /// Convert a [FixedPoint] into metres, from world units.
    pub fn metres(&self) -> Self {
        // FIXME: Prevent duplicate calls to this
        FixedPoint {
            x: (self.x / 65536),
            y: (self.y / 65536),
            z: (self.z / 65536),
        }
    }

    /// Alias for [FixedPoint::metres].
    pub fn meters(&self) -> Self {
        self.metres()
    }
}
