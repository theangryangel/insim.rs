//! Utilities for points in 3D space
#[cfg(feature = "serde")]
use serde::Serialize;

use crate::{Decode, Encode};

#[allow(missing_docs)]
pub trait Pointable: Copy + Clone + Default + Decode + Encode {}

impl Pointable for i32 {}
impl Pointable for f32 {}
impl Pointable for u16 {}

/// A point in 3D space.
#[allow(missing_docs)]
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

impl<T> Decode for Point<T>
where
    T: Pointable,
{
    fn decode(buf: &mut bytes::Bytes) -> Result<Self, crate::Error> {
        let x = T::decode(buf)?;
        let y = T::decode(buf)?;
        let z = T::decode(buf)?;
        Ok(Self { x, y, z })
    }
}

impl<T> Encode for Point<T>
where
    T: Pointable,
{
    fn encode(&self, buf: &mut bytes::BytesMut) -> Result<(), crate::Error> {
        self.x.encode(buf)?;
        self.y.encode(buf)?;
        self.z.encode(buf)?;
        Ok(())
    }
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
