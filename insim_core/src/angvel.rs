//! Angular Velocity
//!
//! AngVel represents the rate of change of heading (angular velocity), normalized internally
//! to radians per second. The LFS protocol encodes this as a signed i16 where:
//! - 16384 = 360 deg/s (clockwise when viewed from above)
//! - 8192 = 180 deg/s (clockwise when viewed from above)
//! - Negative values indicate anticlockwise rotation

use std::fmt;

/// Angular velocity stored as radians per second (f32).
///
/// Represents the rate of change of heading. Positive values indicate clockwise rotation
/// when viewed from above (following the same convention as Direction). Negative values
/// indicate anticlockwise rotation.
/// Internally stored as f32 radians per second for consistency with Direction and Speed.
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct AngVel {
    radians_per_sec: f32,
}

impl AngVel {
    /// AngVel of zero (no rotation).
    pub const ZERO: Self = Self::from_radians_per_sec(0.0);

    /// LFS protocol scale: 16384 = 360 deg/s
    const SCALE: f32 = 360.0 / 16384.0;

    /// Consumes AngVel, returning the inner radian/sec value.
    pub fn into_inner(self) -> f32 {
        self.radians_per_sec
    }

    /// Create AngVel from radians per second.
    pub const fn from_radians_per_sec(value: f32) -> Self {
        Self {
            radians_per_sec: value,
        }
    }

    /// Get the angular velocity in radians per second.
    pub const fn to_radians_per_sec(&self) -> f32 {
        self.radians_per_sec
    }

    /// Create AngVel from degrees per second.
    pub const fn from_degrees_per_sec(value: f32) -> Self {
        Self {
            radians_per_sec: value * std::f32::consts::PI / 180.0,
        }
    }

    /// Get the angular velocity in degrees per second.
    pub fn to_degrees_per_sec(&self) -> f32 {
        self.radians_per_sec * 180.0 / std::f32::consts::PI
    }

    /// Create AngVel from LFS protocol raw i16 value (16384 = 360 deg/s).
    pub fn from_wire_i16(raw: i16) -> Self {
        let degrees_per_sec = raw as f32 * Self::SCALE;
        Self::from_degrees_per_sec(degrees_per_sec)
    }

    /// Get the LFS protocol raw i16 value (16384 = 360 deg/s).
    pub fn to_wire_i16(&self) -> i16 {
        (self.to_degrees_per_sec() / Self::SCALE).round() as i16
    }

    /// Check if this angular velocity represents clockwise rotation (when viewed from above).
    ///
    /// Returns true if the angular velocity is positive (clockwise), false otherwise.
    ///
    /// # Examples
    /// ```
    /// use insim_core::angvel::AngVel;
    /// let clockwise = AngVel::from_degrees_per_sec(90.0);
    /// assert!(clockwise.clockwise());
    ///
    /// let anticlockwise = AngVel::from_degrees_per_sec(-90.0);
    /// assert!(!anticlockwise.clockwise());
    /// ```
    pub fn clockwise(&self) -> bool {
        self.radians_per_sec > 0.0
    }

    /// Check if this angular velocity represents anticlockwise rotation (when viewed from above).
    ///
    /// Returns true if the angular velocity is negative (anticlockwise), false otherwise.
    ///
    /// # Examples
    /// ```
    /// use insim_core::angvel::AngVel;
    /// let anticlockwise = AngVel::from_degrees_per_sec(-90.0);
    /// assert!(anticlockwise.anticlockwise());
    ///
    /// let clockwise = AngVel::from_degrees_per_sec(90.0);
    /// assert!(!clockwise.anticlockwise());
    /// ```
    pub fn anticlockwise(&self) -> bool {
        self.radians_per_sec < 0.0
    }
}

impl Default for AngVel {
    fn default() -> Self {
        Self::ZERO
    }
}

impl fmt::Display for AngVel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:.2} rad/s ({:.2}°/s)",
            self.radians_per_sec,
            self.to_degrees_per_sec()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero() {
        assert_eq!(AngVel::ZERO.to_radians_per_sec(), 0.0);
        assert_eq!(AngVel::ZERO.to_degrees_per_sec(), 0.0);
    }

    #[test]
    fn test_from_radians_per_sec() {
        let av = AngVel::from_radians_per_sec(std::f32::consts::PI);
        assert!((av.to_radians_per_sec() - std::f32::consts::PI).abs() < 0.0001);
    }

    #[test]
    fn test_from_degrees_per_sec() {
        let av = AngVel::from_degrees_per_sec(180.0);
        assert!((av.to_degrees_per_sec() - 180.0).abs() < 0.0001);
    }

    #[test]
    fn test_roundtrip_radians_per_sec() {
        let original = 2.5;
        let av = AngVel::from_radians_per_sec(original);
        assert!((av.to_radians_per_sec() - original).abs() < 0.0001);
    }

    #[test]
    fn test_roundtrip_degrees_per_sec() {
        let original = 45.0;
        let av = AngVel::from_degrees_per_sec(original);
        assert!((av.to_degrees_per_sec() - original).abs() < 0.0001);
    }

    #[test]
    fn test_conversion_radians_to_degrees() {
        // π rad/s = 180 deg/s
        let av = AngVel::from_radians_per_sec(std::f32::consts::PI);
        assert!((av.to_degrees_per_sec() - 180.0).abs() < 0.0001);
    }

    #[test]
    fn test_conversion_degrees_to_radians() {
        // 180 deg/s = π rad/s
        let av = AngVel::from_degrees_per_sec(180.0);
        assert!((av.to_radians_per_sec() - std::f32::consts::PI).abs() < 0.0001);
    }

    #[test]
    fn test_lfs_raw_360_degrees() {
        // 16384 = 360 deg/s
        let av = AngVel::from_wire_i16(16384);
        assert!((av.to_degrees_per_sec() - 360.0).abs() < 0.0001);
    }

    #[test]
    fn test_lfs_raw_180_degrees() {
        // 8192 = 180 deg/s
        let av = AngVel::from_wire_i16(8192);
        assert!((av.to_degrees_per_sec() - 180.0).abs() < 0.0001);
    }

    #[test]
    fn test_lfs_raw_negative() {
        // Negative values indicate anticlockwise rotation
        let av = AngVel::from_wire_i16(-8192);
        assert!((av.to_degrees_per_sec() - (-180.0)).abs() < 0.0001);
    }

    #[test]
    fn test_to_lfs_i16_roundtrip() {
        let original = 12345i16;
        let av = AngVel::from_wire_i16(original);
        let roundtrip = av.to_wire_i16();
        assert_eq!(roundtrip, original);
    }

    #[test]
    fn test_to_lfs_i16_360_degrees() {
        let av = AngVel::from_degrees_per_sec(360.0);
        assert_eq!(av.to_wire_i16(), 16384);
    }

    #[test]
    fn test_to_lfs_i16_180_degrees() {
        let av = AngVel::from_degrees_per_sec(180.0);
        assert_eq!(av.to_wire_i16(), 8192);
    }

    #[test]
    fn test_into_inner() {
        let original = 1.5;
        let av = AngVel::from_radians_per_sec(original);
        assert_eq!(av.into_inner(), original);
    }

    #[test]
    fn test_default() {
        let av = AngVel::default();
        assert_eq!(av.to_radians_per_sec(), 0.0);
    }

    #[test]
    fn test_display() {
        let av = AngVel::from_degrees_per_sec(90.0);
        let display_str = format!("{}", av);
        // Should contain both rad/s and deg/s representations
        assert!(display_str.contains("rad/s"));
        assert!(display_str.contains("°/s"));
    }

    #[test]
    fn test_partial_eq() {
        let av1 = AngVel::from_degrees_per_sec(45.0);
        let av2 = AngVel::from_degrees_per_sec(45.0);
        let av3 = AngVel::from_degrees_per_sec(90.0);

        assert_eq!(av1, av2);
        assert_ne!(av1, av3);
    }

    #[test]
    fn test_partial_ord() {
        let av1 = AngVel::from_degrees_per_sec(45.0);
        let av2 = AngVel::from_degrees_per_sec(90.0);

        assert!(av1 < av2);
        assert!(av2 > av1);
    }

    #[test]
    fn test_clockwise() {
        let clockwise = AngVel::from_degrees_per_sec(90.0);
        assert!(clockwise.clockwise());
        assert!(!clockwise.anticlockwise());
    }

    #[test]
    fn test_anticlockwise() {
        let anticlockwise = AngVel::from_degrees_per_sec(-90.0);
        assert!(anticlockwise.anticlockwise());
        assert!(!anticlockwise.clockwise());
    }

    #[test]
    fn test_clockwise_zero() {
        let zero = AngVel::ZERO;
        assert!(!zero.clockwise());
        assert!(!zero.anticlockwise());
    }
}
