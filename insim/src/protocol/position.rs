//! Utility functions for working with positions.

use deku::prelude::*;

#[cfg(feature = "serde")]
use serde::Serialize;

#[cfg(feature = "uom")]
use crate::units;

#[cfg(feature = "uom")]
use uom;

use insim_core::point::Pointable;

pub struct Point<T>
where
    T: Pointable,
{
    pub x: T,
    pub y: T,
    pub z: T,
}

impl Point<i32> {
    pub fn flipped(&self) -> Self {
        Self {
            x: self.x,
            y: -self.y,
            z: self.z,
        }
    }
}

impl Point<f32> {
    pub fn flipped(&self) -> Self {
        Self {
            x: self.x,
            y: -self.y,
            z: self.z,
        }
    }
}

/// A X, Y, Z position
#[derive(Debug, Eq, PartialEq, DekuRead, DekuWrite, Copy, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[deku(
    ctx = "endian: deku::ctx::Endian",
    ctx_default = "deku::ctx::Endian::Little",
    endian = "endian"
)]
#[cfg(feature = "uom")]
impl<T> Point<T>
where
    T: Pointable + Into<f64>,
{
    pub fn to_uom(
        &self,
    ) -> (
        uom::si::f64::Length,
        uom::si::f64::Length,
        uom::si::f64::Length,
    ) {
        (
            uom::si::f64::Length::new::<units::length::game>(self.x.into()),
            uom::si::f64::Length::new::<units::length::game>(self.y.into()),
            uom::si::f64::Length::new::<units::length::game>(self.z.into()),
        )
    }
}
