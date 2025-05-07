//! Direction
use std::{
    fmt,
    ops::{Add, Div, Mul, Sub},
};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Direction / Heading
pub struct Direction {
    radians: f32,
}

impl Direction {
    /// From game MCI units
    pub fn from_u16_game_units(value: u16) -> Self {
        let radians = (value as f32) * std::f32::consts::PI / 32768.0;
        Self { radians }
    }

    /// From game units (u8)
    pub fn from_u8_game_units(value: u8) -> Self {
        let radians = (value as f32) * std::f32::consts::PI / 128.0;
        Self { radians }
    }

    /// From degrees
    pub fn from_degrees(deg: f32) -> Self {
        Self {
            radians: deg.to_radians(),
        }
        .normalise()
    }

    /// From radians
    pub fn from_radians(rad: f32) -> Self {
        Self { radians: rad }.normalise()
    }

    /// As game units
    pub fn as_u16_game_units(&self) -> u16 {
        ((self.radians * 32768.0 / std::f32::consts::PI)
            .round()
            .clamp(0.0, 65535.0)) as u16
    }

    /// As game units
    pub fn as_u8_game_units(&self) -> u8 {
        ((self.radians * 128.0 / std::f32::consts::PI)
            .round()
            .clamp(0.0, 255.0)) as u8
    }

    /// As degrees
    pub fn as_degrees(&self) -> f32 {
        self.radians.to_degrees()
    }

    /// As radians
    pub fn as_radians(&self) -> f32 {
        self.radians
    }

    /// Normalised Direction
    pub fn normalise(self) -> Self {
        let two_pi = std::f32::consts::TAU; // Same as 2π
        let mut radians = self.radians % two_pi;
        if radians < 0.0 {
            radians += two_pi;
        }
        Self { radians }
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
        .normalise()
    }
}

impl Sub for Direction {
    type Output = Direction;

    fn sub(self, other: Direction) -> Direction {
        Direction {
            radians: self.radians - other.radians,
        }
        .normalise()
    }
}

impl Mul<f32> for Direction {
    type Output = Direction;

    fn mul(self, scalar: f32) -> Direction {
        Direction {
            radians: self.radians * scalar,
        }
        .normalise()
    }
}

impl Div<f32> for Direction {
    type Output = Direction;

    fn div(self, scalar: f32) -> Direction {
        Direction {
            radians: self.radians / scalar,
        }
        .normalise()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_direction_game_units() {
        assert_eq!(Direction::from_u16_game_units(32768).as_degrees(), 180.0);
        assert_eq!(Direction::from_degrees(180.0).as_u16_game_units(), 32768);
    }

    #[test]
    fn test_game_units_u8() {
        assert_eq!(Direction::from_u8_game_units(128).as_degrees(), 180.0);
        assert_eq!(Direction::from_degrees(180.0).as_u8_game_units(), 128);
    }
}
