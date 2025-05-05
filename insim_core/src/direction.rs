//! Direction
use std::{
    fmt,
    ops::{Add, Div, Mul, Sub},
};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
/// Direction / Heading
pub struct Direction {
    radians: f64,
}

impl Direction {
    /// From game
    pub fn from_game_units(value: u16) -> Self {
        let full_circle = 65536.0;
        let radians = (value as f64) / full_circle * 2.0 * std::f64::consts::PI;
        Self { radians }.normalized()
    }

    /// From degrees
    pub fn from_degrees(deg: f64) -> Self {
        Self {
            radians: deg.to_radians(),
        }
        .normalized()
    }

    /// From radians
    pub fn from_radians(rad: f64) -> Self {
        Self { radians: rad }.normalized()
    }

    /// As game units
    pub fn as_game_units(&self) -> u16 {
        let full_circle = 65536.0;
        let units = (self.normalized().radians / (2.0 * std::f64::consts::PI)) * full_circle;
        units.round() as u16
    }

    /// As degrees
    pub fn as_degrees(&self) -> f64 {
        self.radians.to_degrees()
    }

    /// As radians
    pub fn as_radians(&self) -> f64 {
        self.radians
    }

    /// Normalised Direction
    pub fn normalized(self) -> Self {
        let mut r = self.radians % (2.0 * std::f64::consts::PI);
        if r < 0.0 {
            r += 2.0 * std::f64::consts::PI;
        }
        Self { radians: r }
    }
}

impl Default for Direction {
    fn default() -> Self {
        Self { radians: 0.0 }
    }
}

impl fmt::Display for Direction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.2} rad", self.radians)
    }
}

impl Add for Direction {
    type Output = Direction;

    fn add(self, other: Direction) -> Direction {
        Direction {
            radians: self.radians + other.radians,
        }
        .normalized()
    }
}

impl Sub for Direction {
    type Output = Direction;

    fn sub(self, other: Direction) -> Direction {
        Direction {
            radians: self.radians - other.radians,
        }
        .normalized()
    }
}

impl Mul<f64> for Direction {
    type Output = Direction;

    fn mul(self, scalar: f64) -> Direction {
        Direction {
            radians: self.radians * scalar,
        }
        .normalized()
    }
}

impl Div<f64> for Direction {
    type Output = Direction;

    fn div(self, scalar: f64) -> Direction {
        Direction {
            radians: self.radians / scalar,
        }
        .normalized()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_direction_game_units() {
        assert_eq!(Direction::from_game_units(32768).as_degrees(), 180.0);
        assert_eq!(Direction::from_degrees(180.0).as_game_units(), 32768);
    }
}
