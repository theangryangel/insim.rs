//! Utility functions for working with positions.

use deku::prelude::*;
#[cfg(feature = "serde")]
use serde::Serialize;

#[cfg(feature = "uom")]
use crate::units;

#[cfg(feature = "uom")]
use uom;

/// A X, Y, Z position
#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct FixedPoint {
    x: i32,

    y: i32,

    z: i32,
}

impl FixedPoint {
    #[cfg(feature = "uom")]
    pub fn to_uom(
        &self,
    ) -> (
        uom::si::f64::Length,
        uom::si::f64::Length,
        uom::si::f64::Length,
    ) {
        (
            uom::si::f64::Length::new::<units::length::game>(self.x as f64),
            uom::si::f64::Length::new::<units::length::game>(self.y as f64),
            uom::si::f64::Length::new::<units::length::game>(self.z as f64),
        )
    }
}
