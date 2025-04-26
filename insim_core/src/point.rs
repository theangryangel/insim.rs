//! Utilities for points in 3D space
#[cfg(feature = "serde")]
use serde::Serialize;

use crate::ReadWriteBuf;

#[allow(missing_docs)]
pub trait Pointable:
    Copy
    + Clone
    + Default
    + ReadWriteBuf
{
}

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

impl<T> ReadWriteBuf for Point<T>
where
    T: Pointable,
{
    fn read_buf(buf: &mut bytes::Bytes) -> Result<Self, crate::Error> {
        let x = T::read_buf(buf)?;
        let y = T::read_buf(buf)?;
        let z = T::read_buf(buf)?;
        Ok(Self { x, y, z })
    }

    fn write_buf(&self, buf: &mut bytes::BytesMut) -> Result<(), crate::Error> {
        self.x.write_buf(buf)?;
        self.y.write_buf(buf)?;
        self.z.write_buf(buf)?;
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
