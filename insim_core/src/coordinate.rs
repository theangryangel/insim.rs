//! Coordinates / Positional information

use crate::{Decode, Encode};

/// Position in LFS world space.
///
/// - Internal units use fixed-point where 65536 = 1 meter.
/// - Use the `*_metres()` helpers for human-scale values.
/// - Optional `glam` conversions are provided.
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
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
            x: (other.x * Self::SCALE as f64).round() as i32,
            y: (other.y * Self::SCALE as f64).round() as i32,
            z: (other.z * Self::SCALE as f64).round() as i32,
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
            x: (other.x * Self::SCALE as f32).round() as i32,
            y: (other.y * Self::SCALE as f32).round() as i32,
            z: (other.z * Self::SCALE as f32).round() as i32,
        }
    }

    /// Convert to glam Vec3, where xyz are in the internal representation of 65536 = 1m
    #[cfg(feature = "glam")]
    pub fn to_vec3(self) -> glam::Vec3 {
        glam::Vec3::new(self.x as f32, self.y as f32, self.z as f32)
    }

    /// Convert from glam Vec3, where the Vec3 represents 65536 = 1m
    #[cfg(feature = "glam")]
    pub fn from_vec3(other: glam::Vec3) -> Self {
        Self {
            x: other.x as i32,
            y: other.y as i32,
            z: other.z as i32,
        }
    }

    /// Convert to glam IVec3, where xyz are in the internal representation of 65536 = 1m
    #[cfg(feature = "glam")]
    pub fn to_ivec3(self) -> glam::IVec3 {
        glam::IVec3::new(self.x, self.y, self.z)
    }

    /// Convert from glam IVec3, where the Vec3 represents 65536 = 1m
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
    fn decode(ctx: &mut crate::DecodeContext) -> Result<Self, crate::DecodeError> {
        Ok(Self {
            x: ctx.decode::<i32>("x")?,
            y: ctx.decode::<i32>("y")?,
            z: ctx.decode::<i32>("z")?,
        })
    }
}

impl Encode for Coordinate {
    fn encode(&self, ctx: &mut crate::EncodeContext) -> Result<(), crate::EncodeError> {
        ctx.encode("x", &self.x)?;
        ctx.encode("y", &self.y)?;
        ctx.encode("z", &self.z)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Decode, Encode};

    /// Decode -> encode must reproduce the bytes exactly, for every component.
    #[test]
    fn test_roundtrip_bytes_exact() {
        for coord in [
            Coordinate { x: 0, y: 0, z: 0 },
            Coordinate {
                x: i32::MIN,
                y: i32::MAX,
                z: -1,
            },
            Coordinate {
                x: 65536,
                y: -98304,
                z: 1,
            },
        ] {
            let bytes = coord.to_bytes().unwrap();
            let decoded = Coordinate::decode_slice(&bytes).unwrap();
            assert_eq!(coord, decoded, "raw value must survive a byte round-trip");
        }
    }

    /// Re-encoding decoded bytes must yield the identical byte string (idempotent).
    #[test]
    fn test_roundtrip_idempotent() {
        let original: &[u8] = &[
            0x6b, 0x70, 0xfc, 0x00, // x
            0x8e, 0xdc, 0x47, 0x00, // y
            0x10, 0x27, 0x00, 0x00, // z
        ];
        let decoded = Coordinate::decode_slice(original).unwrap();
        let reencoded = decoded.to_bytes().unwrap();
        assert_eq!(&reencoded[..], original);
    }

    #[cfg(feature = "glam")]
    #[test]
    fn test_from_dvec3_metres_does_not_truncate() {
        // 1.5 m must scale to 1.5 * 65536 = 98304, not 65536 (the old truncating bug).
        let c = Coordinate::from_dvec3_metres(glam::DVec3::new(1.5, -2.25, 0.5));
        assert_eq!(c.x, 98304);
        assert_eq!(c.y, -147456);
        assert_eq!(c.z, 32768);
    }

    #[cfg(feature = "glam")]
    #[test]
    fn test_from_vec3_metres_does_not_truncate() {
        let c = Coordinate::from_vec3_metres(glam::Vec3::new(1.5, -2.25, 0.5));
        assert_eq!(c.x, 98304);
        assert_eq!(c.y, -147456);
        assert_eq!(c.z, 32768);
    }

    #[cfg(feature = "glam")]
    #[test]
    fn test_metres_roundtrip_dvec3() {
        // Values chosen to be exactly representable so the round-trip is lossless.
        let original = glam::DVec3::new(12.5, -340.25, 7.125);
        let c = Coordinate::from_dvec3_metres(original);
        let back = c.to_dvec3_metres();
        assert_eq!(back, original);
    }
}
