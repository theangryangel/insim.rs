//! Coordinates / Positional information

use crate::{Decode, Encode};

/// Position in LFS world space.
///
/// - Internal units use fixed-point where 65536 = 1 meter.
/// - Use the `*_metres()` helpers for human-scale values.
/// - Optional `glam` conversions are provided.
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Coordinate {
    /// X coordinate in internal units.
    pub x: i32,
    /// Y coordinate in internal units.
    pub y: i32,
    /// Z coordinate in internal units.
    pub z: i32,
}

impl Coordinate {
    // Scale for 1m to internal representation
    const SCALE: i32 = 65536;

    /// X (in metres)
    pub fn x_metres(&self) -> f32 {
        self.x as f32 / Self::SCALE as f32
    }

    /// Y (in metres)
    pub fn y_metres(&self) -> f32 {
        self.y as f32 / Self::SCALE as f32
    }

    /// Z (in metres)
    pub fn z_metres(&self) -> f32 {
        self.z as f32 / Self::SCALE as f32
    }

    /// X, Y, Z (in metres)
    pub fn xyz_metres(&self) -> (f32, f32, f32) {
        (self.x_metres(), self.y_metres(), self.z_metres())
    }

    /// Convert to glam Vec3, where xyz are in metres
    #[cfg(feature = "glam")]
    pub fn to_vec3_metres(self) -> glam::Vec3 {
        glam::Vec3::new(
            self.x as f32 / Self::SCALE as f32,
            self.y as f32 / Self::SCALE as f32,
            self.z as f32 / Self::SCALE as f32,
        )
    }

    /// Convert from glam Vec3, where the Vec3 is in metres
    #[cfg(feature = "glam")]
    pub fn from_dvec3_metres(other: glam::DVec3) -> Self {
        Self {
            x: other.x as i32 * Self::SCALE,
            y: other.y as i32 * Self::SCALE,
            z: other.z as i32 * Self::SCALE,
        }
    }

    /// Convert to glam Vec3, where xyz are in metres
    #[cfg(feature = "glam")]
    pub fn to_dvec3_metres(self) -> glam::DVec3 {
        glam::DVec3::new(
            self.x as f64 / Self::SCALE as f64,
            self.y as f64 / Self::SCALE as f64,
            self.z as f64 / Self::SCALE as f64,
        )
    }

    /// Convert from glam Vec3, where the Vec3 is in metres
    #[cfg(feature = "glam")]
    pub fn from_vec3_metres(other: glam::Vec3) -> Self {
        Self {
            x: other.x as i32 * Self::SCALE,
            y: other.y as i32 * Self::SCALE,
            z: other.z as i32 * Self::SCALE,
        }
    }

    /// Convert to glam Vec3, where xyz are in the internal representation of 63336 = 1m
    #[cfg(feature = "glam")]
    pub fn to_vec3(self) -> glam::Vec3 {
        glam::Vec3::new(self.x as f32, self.y as f32, self.z as f32)
    }

    /// Convert from glam Vec3, where the Vec3 represents 63336 = 1m
    #[cfg(feature = "glam")]
    pub fn from_vec3(other: glam::Vec3) -> Self {
        Self {
            x: other.x as i32,
            y: other.y as i32,
            z: other.z as i32,
        }
    }

    /// Convert to glam IVec3, where xyz are in the internal representation of 63336 = 1m
    #[cfg(feature = "glam")]
    pub fn to_ivec3(self) -> glam::IVec3 {
        glam::IVec3::new(self.x, self.y, self.z)
    }

    /// Convert from glam IVec3, where the Vec3 represents 63336 = 1m
    #[cfg(feature = "glam")]
    pub fn from_ivec3(other: glam::IVec3) -> Self {
        Self {
            x: other.x,
            y: other.y,
            z: other.z,
        }
    }
}

impl Decode for Coordinate {
    fn decode(buf: &mut bytes::Bytes) -> Result<Self, crate::DecodeError> {
        Ok(Self {
            x: i32::decode(buf)?,
            y: i32::decode(buf)?,
            z: i32::decode(buf)?,
        })
    }
}

impl Encode for Coordinate {
    fn encode(&self, buf: &mut bytes::BytesMut) -> Result<(), crate::EncodeError> {
        self.x.encode(buf)?;
        self.y.encode(buf)?;
        self.z.encode(buf)?;
        Ok(())
    }
}
