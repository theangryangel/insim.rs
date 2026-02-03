//! General purpose Vector

use crate::{Decode, Encode};

/// 3D vector used for motion and forces.
///
/// - Units depend on context (velocity, acceleration, direction).
/// - Use the optional `glam` conversions when enabled.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Vector(pub f32, pub f32, pub f32);

impl Vector {
    #[cfg(feature = "glam")]
    /// Convert to glam Vec3
    pub fn to_vec3(self) -> glam::Vec3 {
        glam::Vec3::new(self.0, self.1, self.2)
    }
}

#[cfg(feature = "glam")]
impl From<Vector> for glam::Vec3 {
    fn from(vector: Vector) -> Self {
        glam::Vec3::new(vector.0, vector.1, vector.2)
    }
}

impl Decode for Vector {
    fn decode(buf: &mut bytes::Bytes) -> Result<Self, crate::DecodeError> {
        Ok(Self(
            f32::decode(buf)?,
            f32::decode(buf)?,
            f32::decode(buf)?,
        ))
    }
}

impl Encode for Vector {
    fn encode(&self, buf: &mut bytes::BytesMut) -> Result<(), crate::EncodeError> {
        self.0.encode(buf)?;
        self.1.encode(buf)?;
        self.2.encode(buf)?;
        Ok(())
    }
}
