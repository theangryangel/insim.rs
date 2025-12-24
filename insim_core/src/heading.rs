//! Heading
use std::fmt;

/// Angular measurement stored as radians (f64).
///
/// Represents heading or direction in LFS game space. Internally stored as f64 radians
/// for consistency across the codebase, with convenient conversions to/from degrees.
/// Uses double precision to minimize cumulative errors in physics calculations and
/// integration over time.
///
/// # Orientation
/// - 0 = world Y direction** (forward/north)
/// - 90 = world X direction** (right/east)
/// - 180 = -Y direction** (backward/south)
/// - 270 = -X direction** (left/west)
///
/// Angles increase in the anti-clockwise direction when viewed from above.
///
/// # Wraparound
/// Angles are **not** automatically normalized. A Heading created from 450 will
/// preserve that value internally. Use [`normalize()`](Heading::normalize) to wrap
/// angles to the 0->360 range if needed.
///
/// ```
/// use insim_core::heading::Heading;
///
/// let over = Heading::from_degrees(450.0);
/// assert!((over.to_degrees() - 450.0).abs() < 0.0001);  // Preserved as-is
///
/// let normalized = over.normalize();
/// assert!((normalized.to_degrees() - 90.0).abs() < 0.0001);  // Wrapped to [0, 360)
/// ```
///
/// # Examples
/// ```
/// use insim_core::heading::Heading;
///
/// let forward = Heading::from_degrees(0.0);    // Y direction
/// let right = Heading::from_degrees(90.0);     // X direction
/// let backward = Heading::from_degrees(180.0); // -Y direction
///
/// assert_eq!(forward.to_degrees(), 0.0);
/// assert_eq!(right.to_degrees(), 90.0);
/// ```
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Heading {
    radians: f64,
}

impl Heading {
    /// Heading pointing along world Y (0).
    ///
    /// This is typically north/forward in LFS game coordinates.
    pub const ZERO: Self = Self::from_radians(0.0);

    /// Heading pointing north (world Y direction, 0).
    pub const NORTH: Self = Self::from_radians(0.0);

    /// Heading pointing east (world X direction, 90).
    pub const EAST: Self = Self::from_radians(std::f64::consts::FRAC_PI_2);

    /// Heading pointing south (opposite of world Y, 180).
    pub const SOUTH: Self = Self::from_radians(std::f64::consts::PI);

    /// Heading pointing west (opposite of world X, 270).
    pub const WEST: Self = Self::from_radians(3.0 * std::f64::consts::FRAC_PI_2);

    /// Consumes Heading, returning the inner radian value.
    pub fn into_inner(self) -> f64 {
        self.radians
    }

    /// Create Heading from radians.
    ///
    /// 0 radians = world Y direction (forward)
    /// π/2 radians = world X direction (right)
    /// π radians = -Y direction (backward)
    /// 3π/2 radians = -X direction (left)
    pub const fn from_radians(value: f64) -> Self {
        Self { radians: value }
    }

    /// Get the angle in radians.
    ///
    /// 0 radians = world Y direction (forward)
    /// π/2 radians = world X direction (right)
    /// π radians = -Y direction (backward)
    /// 3π/2 radians = -X direction (left)
    pub const fn to_radians(&self) -> f64 {
        self.radians
    }

    /// Create Heading from degrees.
    ///
    /// 0 = world Y direction (forward)
    /// 90 = world X direction (right)
    /// 180 = -Y direction (backward)
    /// 270 = -X direction (left)
    pub const fn from_degrees(value: f64) -> Self {
        Self {
            radians: value * std::f64::consts::PI / 180.0,
        }
    }

    /// Get the angle in degrees.
    ///
    /// 0 = world Y direction (forward)
    /// 90 = world X direction (right)
    /// 180 = -Y direction (backward)
    /// 270 = -X direction (left)
    pub const fn to_degrees(&self) -> f64 {
        self.radians * 180.0 / std::f64::consts::PI
    }

    /// Normalize the angle to the range 0->360.
    ///
    /// Useful when you need to compare directions or ensure angles are in a
    /// canonical form.
    ///
    /// # Examples
    /// ```
    /// use insim_core::heading::Heading;
    ///
    /// // Over-rotation wraps back
    /// let over = Heading::from_degrees(450.0);
    /// assert!((over.normalize().to_degrees() - 90.0).abs() < 0.0001);
    ///
    /// // Negative angles wrap forward
    /// let negative = Heading::from_degrees(-90.0);
    /// assert!((negative.normalize().to_degrees() - 270.0).abs() < 0.0001);
    ///
    /// // Already normalized angles unchanged
    /// let normal = Heading::from_degrees(45.0);
    /// assert!((normal.normalize().to_degrees() - 45.0).abs() < 0.0001);
    /// ```
    pub fn normalize(&self) -> Heading {
        let degrees = self.to_degrees();
        let normalized = degrees % 360.0;
        let normalized = if normalized < 0.0 {
            normalized + 360.0
        } else {
            normalized
        };
        Heading::from_degrees(normalized)
    }

    /// Get the opposite direction (180 rotation).
    ///
    /// Returns the direction that is directly opposite, i.e., rotated 180.
    ///
    /// # Examples
    /// ```
    /// use insim_core::heading::Heading;
    ///
    /// let north = Heading::NORTH;
    /// let south = north.opposite();
    /// assert!((south.to_degrees() - 180.0).abs() < 0.0001);
    ///
    /// let east = Heading::EAST;
    /// let west = east.opposite();
    /// assert!((west.to_degrees() - 270.0).abs() < 0.0001);
    /// ```
    pub fn opposite(&self) -> Heading {
        Heading::from_radians(self.radians + std::f64::consts::PI)
    }

    /// Convert from LFS object heading u8 to Heading.
    ///
    /// LFS encodes object headings as a u8 where 360 degrees is represented in 256 values:
    /// Heading = (heading_in_degrees + 180) * 256 / 360
    ///
    /// Therefore: heading_in_degrees = (heading * 360 / 256) - 180
    ///
    /// Examples:
    /// - 128 = 0 (world Y direction / forward)
    /// - 192 = 90 (world X direction / right)
    /// - 0 = 180 (opposite of world Y direction / backward)
    /// - 64 = -90 (opposite of world X direction / left)
    pub const fn from_objectinfo_wire(heading: u8) -> Self {
        let degrees = (heading as f64) * (360.0 / 256.0) - 180.0;
        Self::from_degrees(degrees)
    }

    /// Convert Heading to LFS object heading u8.
    ///
    /// Returns a u8 in the range 0-255 representing the heading.
    /// Uses the formula: Heading = (heading_in_degrees + 180) * 256 / 360
    pub fn to_objectinfo_wire(&self) -> u8 {
        let mut value = ((self.to_degrees() + 180.0) * 256.0 / 360.0).round() as i32;
        // Handle wraparound: 256 should map to 0
        if value == 256 {
            value = 0;
        }
        value as u8
    }
}

impl fmt::Display for Heading {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.2}", self.to_degrees())
    }
}

impl Default for Heading {
    fn default() -> Self {
        Self::ZERO
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_radians() {
        assert_eq!(
            Heading::from_radians(std::f64::consts::PI).to_degrees(),
            180.0
        );
    }

    #[test]
    fn test_degrees() {
        assert!((Heading::from_degrees(180.0).to_radians() - std::f64::consts::PI).abs() < 0.001);
    }

    #[test]
    fn test_zero() {
        assert_eq!(Heading::ZERO.to_degrees(), 0.0);
        assert_eq!(Heading::ZERO.to_radians(), 0.0);
    }

    #[test]
    fn test_roundtrip_degrees() {
        let original = 45.0;
        let direction = Heading::from_degrees(original);
        assert!((direction.to_degrees() - original).abs() < 0.0001);
    }

    #[test]
    fn test_normalize_over_rotation() {
        let over = Heading::from_degrees(450.0);
        let normalized = over.normalize();
        assert!((normalized.to_degrees() - 90.0).abs() < 0.0001);
    }

    #[test]
    fn test_normalize_negative() {
        let negative = Heading::from_degrees(-90.0);
        let normalized = negative.normalize();
        assert!((normalized.to_degrees() - 270.0).abs() < 0.0001);
    }

    #[test]
    fn test_normalize_already_normal() {
        let normal = Heading::from_degrees(45.0);
        let normalized = normal.normalize();
        assert!((normalized.to_degrees() - 45.0).abs() < 0.0001);
    }

    #[test]
    fn test_normalize_zero() {
        let zero = Heading::from_degrees(0.0);
        let normalized = zero.normalize();
        assert!((normalized.to_degrees() - 0.0).abs() < 0.0001);
    }

    #[test]
    fn test_normalize_360() {
        let full_rotation = Heading::from_degrees(360.0);
        let normalized = full_rotation.normalize();
        assert!((normalized.to_degrees() - 0.0).abs() < 0.0001);
    }

    #[test]
    fn test_normalize_large_negative() {
        let large_negative = Heading::from_degrees(-450.0);
        let normalized = large_negative.normalize();
        assert!((normalized.to_degrees() - 270.0).abs() < 0.0001);
    }

    #[test]
    fn test_no_auto_normalize() {
        // Verify that construction doesn't auto-normalize
        let over = Heading::from_degrees(450.0);
        assert!((over.to_degrees() - 450.0).abs() < 0.0001);
    }

    #[test]
    fn test_from_u8_heading() {
        // Test conversion from u8 heading using formula:
        // heading_in_degrees = (heading * 360 / 256) - 180
        let forward = Heading::from_objectinfo_wire(128);
        assert!((forward.to_degrees() - 0.0).abs() < 0.0001);

        let backward = Heading::from_objectinfo_wire(0);
        // heading 0 gives -180, which is equivalent to 180 (differ by 360)
        assert!((backward.to_degrees() - (-180.0)).abs() < 0.0001);

        let right = Heading::from_objectinfo_wire(192);
        assert!((right.to_degrees() - 90.0).abs() < 0.0001);

        let left = Heading::from_objectinfo_wire(64);
        assert!((left.to_degrees() - (-90.0)).abs() < 0.0001);
    }

    #[test]
    fn test_to_u8_heading() {
        // Test roundtrip conversion
        let original = Heading::from_degrees(45.0);
        let heading = original.to_objectinfo_wire();
        let restored = Heading::from_objectinfo_wire(heading);
        assert!((original.to_degrees() - restored.to_degrees()).abs() < 0.2);

        // Test specific values using formula: Heading = (heading_in_degrees + 180) * 256 / 360
        assert_eq!(Heading::from_degrees(0.0).to_objectinfo_wire(), 128);
        // 180 wraps around: (180 + 180) * 256 / 360 = 256 -> 0
        assert_eq!(Heading::from_degrees(180.0).to_objectinfo_wire(), 0);
        assert_eq!(Heading::from_degrees(90.0).to_objectinfo_wire(), 192);
        assert_eq!(Heading::from_degrees(-90.0).to_objectinfo_wire(), 64);
    }

    #[test]
    fn test_cardinal_directions() {
        assert!((Heading::NORTH.to_degrees() - 0.0).abs() < 0.0001);
        assert!((Heading::EAST.to_degrees() - 90.0).abs() < 0.0001);
        assert!((Heading::SOUTH.to_degrees() - 180.0).abs() < 0.0001);
        assert!((Heading::WEST.to_degrees() - 270.0).abs() < 0.0001);
    }

    #[test]
    fn test_opposite() {
        let north = Heading::NORTH;
        let south = north.opposite();
        assert!((south.to_degrees() - 180.0).abs() < 0.0001);

        let east = Heading::EAST;
        let west = east.opposite();
        assert!((west.to_degrees() - 270.0).abs() < 0.0001);
    }

    #[test]
    fn test_opposite_roundtrip() {
        let original = Heading::from_degrees(45.0);
        let twice_opposite = original.opposite().opposite();
        // Note: Heading doesn't auto-normalize, so opposite().opposite() adds 360
        assert!((twice_opposite.to_degrees() - (original.to_degrees() + 360.0)).abs() < 0.0001);
    }

    #[test]
    fn test_opposite_arbitrary_angle() {
        let dir = Heading::from_degrees(123.45);
        let opp = dir.opposite();
        let expected = (123.45 + 180.0) % 360.0;
        assert!((opp.to_degrees() - expected).abs() < 0.0001);
    }
}
