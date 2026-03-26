//! General purpose Vector

use crate::{Decode, Encode};

/// 3D vector used for motion and forces.
///
/// - Units depend on context (velocity, acceleration, direction).
/// - Use the optional `glam` conversions when enabled.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
    fn decode(ctx: &mut crate::DecodeContext) -> Result<Self, crate::DecodeError> {
        Ok(Self(
            ctx.decode::<f32>("0")?,
            ctx.decode::<f32>("1")?,
            ctx.decode::<f32>("2")?,
        ))
    }
}

impl Encode for Vector {
    fn encode(&self, ctx: &mut crate::EncodeContext) -> Result<(), crate::EncodeError> {
        ctx.encode("0", &self.0)?;
        ctx.encode("1", &self.1)?;
        ctx.encode("2", &self.2)
    }
}
