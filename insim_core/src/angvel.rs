//! Angular velocity wire type.
//!
//! LFS (in `Mci`'s `CompCar`) encodes angular velocity as a signed `i16` where
//! 16384 = 360°/s. Rather than decode to a lossy floating-point value and convert
//! back (which is not idempotent on the wire), [`AngVelI16`] stores the **raw
//! integer** and exposes degree/radian accessors on demand. Decoding then encoding
//! reproduces the original `i16` exactly.
//!
//! - 16384 = 360°/s, 8192 = 180°/s.
//! - Positive values are clockwise viewed from above; negative anticlockwise.
//!
//! Note: OutSim transmits angular velocity as a [`Vector`](crate::vector::Vector)
//! of real `f32`s, not this fixed-point form.

use std::fmt;

use crate::{Decode, Encode};

/// Angular velocity as transmitted in `CompCar` (MCI): a signed `i16` where
/// 16384 = 360°/s.
///
/// Stores the raw wire value; conversions to degrees/radians per second are
/// accessors. Decoding then encoding reproduces the original `i16` exactly.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct AngVelI16(i16);

impl AngVelI16 {
    /// Zero angular velocity.
    pub const ZERO: Self = Self(0);

    /// Raw wire units per 360°/s.
    const UNITS_PER_TURN_PER_SEC: f32 = 16384.0;

    /// Construct from the raw wire value (lossless).
    pub const fn from_raw(raw: i16) -> Self {
        Self(raw)
    }

    /// The raw wire value (lossless).
    pub const fn to_raw(self) -> i16 {
        self.0
    }

    /// Consumes `self`, returning the raw inner value (same as [`to_raw`](Self::to_raw)).
    pub const fn into_inner(self) -> i16 {
        self.0
    }

    /// Construct from degrees per second (rounded to the nearest wire unit).
    pub fn from_degrees_per_sec(value: f32) -> Self {
        let units = (value / 360.0 * Self::UNITS_PER_TURN_PER_SEC).round();
        Self(units.clamp(i16::MIN as f32, i16::MAX as f32) as i16)
    }

    /// Angular velocity in degrees per second.
    pub fn to_degrees_per_sec(self) -> f32 {
        self.0 as f32 / Self::UNITS_PER_TURN_PER_SEC * 360.0
    }

    /// Construct from radians per second.
    pub fn from_radians_per_sec(value: f32) -> Self {
        Self::from_degrees_per_sec(value.to_degrees())
    }

    /// Angular velocity in radians per second.
    pub fn to_radians_per_sec(self) -> f32 {
        self.to_degrees_per_sec().to_radians()
    }

    /// Whether this represents clockwise rotation (viewed from above), i.e. the
    /// raw value is positive.
    pub const fn clockwise(self) -> bool {
        self.0 > 0
    }

    /// Whether this represents anticlockwise rotation (viewed from above), i.e.
    /// the raw value is negative.
    pub const fn anticlockwise(self) -> bool {
        self.0 < 0
    }

    /// Whether this is exactly zero.
    pub const fn is_zero(self) -> bool {
        self.0 == 0
    }
}

impl fmt::Display for AngVelI16 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:.2} rad/s ({:.2}°/s)",
            self.to_radians_per_sec(),
            self.to_degrees_per_sec()
        )
    }
}

impl Decode for AngVelI16 {
    fn decode(ctx: &mut crate::DecodeContext) -> Result<Self, crate::DecodeError> {
        Ok(Self(ctx.decode::<i16>("angvel")?))
    }
}

impl Encode for AngVelI16 {
    fn encode(&self, ctx: &mut crate::EncodeContext) -> Result<(), crate::EncodeError> {
        ctx.encode("angvel", &self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_raw_roundtrip_exact() {
        for raw in [i16::MIN, -8192, -1, 0, 1, 8192, 16384, i16::MAX] {
            assert_eq!(AngVelI16::from_raw(raw).to_raw(), raw);
        }
    }

    #[test]
    fn test_degrees_per_sec() {
        // 16384 = 360°/s, 8192 = 180°/s
        assert_eq!(AngVelI16::from_raw(16384).to_degrees_per_sec(), 360.0);
        assert_eq!(AngVelI16::from_raw(8192).to_degrees_per_sec(), 180.0);
        assert_eq!(AngVelI16::from_raw(-8192).to_degrees_per_sec(), -180.0);

        assert_eq!(AngVelI16::from_degrees_per_sec(360.0).to_raw(), 16384);
        assert_eq!(AngVelI16::from_degrees_per_sec(180.0).to_raw(), 8192);
    }

    #[test]
    fn test_radians_per_sec() {
        let av = AngVelI16::from_raw(8192);
        assert!((av.to_radians_per_sec() - std::f32::consts::PI).abs() < 1e-4);
    }

    #[test]
    fn test_clockwise() {
        assert!(AngVelI16::from_raw(90).clockwise());
        assert!(!AngVelI16::from_raw(90).anticlockwise());
        assert!(AngVelI16::from_raw(-90).anticlockwise());
        assert!(!AngVelI16::ZERO.clockwise());
        assert!(!AngVelI16::ZERO.anticlockwise());
    }

    #[test]
    fn test_default_and_zero() {
        assert!(AngVelI16::default().is_zero());
        assert_eq!(AngVelI16::default(), AngVelI16::ZERO);
    }
}
