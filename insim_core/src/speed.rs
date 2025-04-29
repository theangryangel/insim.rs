//! Utilities for speed
use std::{
    fmt,
    ops::{Add, Div, Mul, Sub},
};

/// Representation of Speed, stored meters per second internally, with conversions to common units.
/// Implementation inspired by Duration from the standard library.
/// The intention is to keep things simple.
/// There is no intention to create a complex uom-style system.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Speed {
    // meters per second
    inner: f64,
}

impl Speed {
    /// From game units: 32768 = 100 m/s
    pub fn from_game_mci_units(value: u16) -> Self {
        Self {
            inner: (value as f64) / 327.68,
        }
    }

    /// From game closing speed: 10 = 1m/s
    pub fn from_game_closing_speed(value: u16) -> Self {
        Self {
            inner: (value as f64) / 10.0,
        }
    }

    /// From meters per second
    pub fn from_meters_per_sec(value: f64) -> Self {
        Self { inner: value }
    }

    /// From Km per hour
    pub fn from_kilometers_per_hour(value: f64) -> Self {
        Self { inner: value / 3.6 }
    }

    /// From miles per hour
    pub fn from_miles_per_hour(value: f64) -> Self {
        Self {
            inner: value * 0.44704,
        }
    }

    /// As game units
    pub fn as_game_mci_units(&self) -> u16 {
        (self.inner * 327.68) as u16
    }

    /// as game closing speed: 10 = 1m/s
    pub fn as_game_closing_speed(&self) -> u16 {
        (self.inner * 10.0) as u16
    }

    /// As meters per second
    pub fn as_meters_per_sec(&self) -> f64 {
        self.inner
    }

    /// As Km per hour
    pub fn as_kilometers_per_hour(&self) -> f64 {
        self.inner * 3.6
    }

    /// As miles per hour
    pub fn as_miles_per_hour(&self) -> f64 {
        self.inner / 0.44704
    }
}

impl fmt::Display for Speed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.2} m/s", self.inner)
    }
}

impl Default for Speed {
    fn default() -> Self {
        Self { inner: 0.0 }
    }
}

impl Add for Speed {
    type Output = Speed;

    fn add(self, other: Speed) -> Speed {
        Speed {
            inner: self.inner + other.inner,
        }
    }
}

impl Sub for Speed {
    type Output = Speed;

    fn sub(self, other: Speed) -> Speed {
        Speed {
            inner: self.inner - other.inner,
        }
    }
}

impl Mul<f64> for Speed {
    type Output = Speed;

    fn mul(self, scalar: f64) -> Speed {
        Speed {
            inner: self.inner * scalar,
        }
    }
}

impl Div<f64> for Speed {
    type Output = Speed;

    fn div(self, scalar: f64) -> Speed {
        Speed {
            inner: self.inner / scalar,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_game_units() {
        assert_eq!(Speed::from_game_mci_units(32768).as_meters_per_sec(), 100.0);
        assert_eq!(Speed::from_meters_per_sec(100.0).as_game_mci_units(), 32768);
    }
}
