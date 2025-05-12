//! Utilities for points in 3D space

use num_traits::{FromPrimitive, Num, ToPrimitive};

use crate::{Decode, Encode};

/// A point in 3D space.
#[allow(missing_docs)]
#[derive(Default, Copy, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Point<T>
where
    T: Copy + Num + ToPrimitive + FromPrimitive + Decode + Encode,
{
    pub x: T,
    pub y: T,
    pub z: T,
}

impl<T> Point<T>
where
    T: Copy + Num + ToPrimitive + FromPrimitive + Decode + Encode,
{
    /// Euclidean distance between 2 points
    pub fn distance(&self, other: &Point<T>) -> f32 {
        let dx = self.x.to_f32().unwrap_or(0.0) - other.x.to_f32().unwrap_or(0.0);
        let dy = self.y.to_f32().unwrap_or(0.0) - other.y.to_f32().unwrap_or(0.0);
        let dz = self.z.to_f32().unwrap_or(0.0) - other.z.to_f32().unwrap_or(0.0);
        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    /// Dot product
    pub fn dot(&self, other: &Point<T>) -> f32 {
        let x = self.x.to_f32().unwrap_or(0.0) * other.x.to_f32().unwrap_or(0.0);
        let y = self.y.to_f32().unwrap_or(0.0) * other.y.to_f32().unwrap_or(0.0);
        let z = self.z.to_f32().unwrap_or(0.0) * other.z.to_f32().unwrap_or(0.0);
        x + y + z
    }

    /// Project onto a segment
    pub fn project_onto_segment(&self, a: &Point<T>, b: &Point<T>) -> Point<T> {
        let p = self;
        let ab = Point {
            x: b.x - a.x,
            y: b.y - a.y,
            z: b.z - a.z,
        };
        let ap = Point {
            x: p.x - a.x,
            y: p.y - a.y,
            z: p.z - a.z,
        };

        let ab_len2 = ab.dot(&ab);
        if ab_len2 == 0.0 {
            return *a;
        }

        let t = ap.dot(&ab) / ab_len2;
        let t = t.clamp(0.0, 1.0);

        Point {
            x: T::from_f32(
                (a.x.to_f32().unwrap_or(0.0) + ab.x.to_f32().unwrap_or(0.0) * t).round(),
            )
            .unwrap(),
            y: T::from_f32(
                (a.y.to_f32().unwrap_or(0.0) + ab.y.to_f32().unwrap_or(0.0) * t).round(),
            )
            .unwrap(),
            z: T::from_f32(
                (a.z.to_f32().unwrap_or(0.0) + ab.z.to_f32().unwrap_or(0.0) * t).round(),
            )
            .unwrap(),
        }
    }

    /// Project onto a segment, returns ratio
    pub fn project_onto_segment_ratio(&self, a: &Point<T>, b: &Point<T>) -> f32 {
        let p = self;
        let ab = Point {
            x: b.x - a.x,
            y: b.y - a.y,
            z: b.z - a.z,
        };
        let ap = Point {
            x: p.x - a.x,
            y: p.y - a.y,
            z: p.z - a.z,
        };

        let ab_len2 = ab.dot(&ab);
        if ab_len2 == 0.0 {
            return 0.0;
        }

        (ap.dot(&ab) / ab_len2).clamp(0.0, 1.0)
    }
}

impl<T> Decode for Point<T>
where
    T: Copy + Num + ToPrimitive + FromPrimitive + Decode + Encode,
{
    fn decode(buf: &mut bytes::Bytes) -> Result<Self, crate::DecodeError> {
        let x = T::decode(buf)?;
        let y = T::decode(buf)?;
        let z = T::decode(buf)?;
        Ok(Self { x, y, z })
    }
}

impl<T> Encode for Point<T>
where
    T: Copy + Num + ToPrimitive + FromPrimitive + Decode + Encode,
{
    fn encode(&self, buf: &mut bytes::BytesMut) -> Result<(), crate::EncodeError> {
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
