//! Utilities for speed
use std::fmt;

/// Speed stored as meters per second.
///
/// - Internal units are m/s.
/// - Helper methods convert to and from kph and mph.
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Speed {
    inner: f32,
}

impl Speed {
    /// Zero m/s
    pub const ZERO: Self = Self::from_meters_per_sec(0.0);

    /// Consumes Speed, returning the raw wrapped value.
    pub fn into_inner(self) -> f32 {
        self.inner
    }

    /// Convert from meters per sec
    pub const fn from_meters_per_sec(value: f32) -> Self {
        Self { inner: value }
    }
    /// Convert into meters per sec
    pub const fn to_meters_per_sec(&self) -> f32 {
        self.inner
    }

    /// Convert from kph
    pub const fn from_kilometers_per_hour(value: f32) -> Self {
        Self { inner: value / 3.6 }
    }
    /// Convert into kph
    pub const fn to_kilometers_per_hour(&self) -> f32 {
        self.inner * 3.6
    }

    /// Convert from mph
    pub const fn from_miles_per_hour(value: f32) -> Self {
        Self {
            inner: value / 2.23694,
        }
    }

    /// Convert into mph
    pub const fn to_miles_per_hour(&self) -> f32 {
        self.inner * 2.23694
    }

    /// Check if speed is zero.
    pub const fn is_zero(&self) -> bool {
        self.inner == 0.0
    }
}

impl fmt::Display for Speed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.2}m/s", self.inner)
    }
}

impl Default for Speed {
    fn default() -> Self {
        Self::ZERO
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_mps() {
        assert_eq!(Speed::from_meters_per_sec(100.0).to_meters_per_sec(), 100.0);

        assert_eq!(Speed::from_meters_per_sec(100.0).into_inner(), 100.0);
    }

    #[test]
    fn test_kmph() {
        assert_eq!(
            Speed::from_kilometers_per_hour(100.0).into_inner(),
            27.777779
        );
    }

    #[test]
    fn test_mph_kmph() {
        assert_eq!(
            Speed::from_kilometers_per_hour(100.0).to_miles_per_hour(),
            62.137222
        );
    }

    #[test]
    fn test_is_zero() {
        assert!(Speed::ZERO.is_zero());
        assert!(Speed::from_meters_per_sec(0.0).is_zero());
        assert!(!Speed::from_meters_per_sec(0.1).is_zero());
        assert!(!Speed::from_meters_per_sec(100.0).is_zero());
    }
}
