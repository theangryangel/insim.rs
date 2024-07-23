//! Utilities for points in 3D space
use binrw::{binrw, BinRead, BinWrite};
#[cfg(feature = "serde")]
use serde::Serialize;

#[allow(missing_docs)]
pub trait Pointable:
    Copy + Clone + Default + for<'a> BinRead<Args<'a> = ()> + for<'a> BinWrite<Args<'a> = ()>
{
}

impl Pointable for i32 {}
impl Pointable for f32 {}
impl Pointable for u16 {}

/// A point in 3D space.
#[allow(missing_docs)]
#[binrw]
#[derive(Default, Copy, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct Point<T>
where
    T: Pointable,
{
    pub x: T,
    pub y: T,
    pub z: T,
}

impl Point<i32> {
    /// Flip the Y axis
    pub fn flipped(&self) -> Self {
        Self {
            x: self.x,
            y: -self.y,
            z: self.z,
        }
    }
}

impl Point<f32> {
    /// Flip the Y axis
    pub fn flipped(&self) -> Self {
        Self {
            x: self.x,
            y: -self.y,
            z: self.z,
        }
    }
}
