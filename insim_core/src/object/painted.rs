//! Painted objects
use std::convert::TryFrom;

use super::{ObjectVariant, ObjectWire};
use crate::{DecodeError, direction::Direction};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[repr(u8)]
#[allow(missing_docs)]
#[non_exhaustive]
/// Paint Colour
pub enum PaintColour {
    #[default]
    White = 0,
    Yellow = 1,
}

impl From<u8> for PaintColour {
    fn from(value: u8) -> Self {
        match value & 0x01 {
            0 => Self::White,
            1 => Self::Yellow,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[repr(u8)]
#[allow(missing_docs)]
#[non_exhaustive]
/// Letter / Character
pub enum Character {
    #[default]
    A = 0,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    DigL,
    DigR,
    DigU,
    DigD,
    Hash,
    At,
    Zero,
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Dot,
    Colon,
    Slash,
    LParen,
    RParen,
    Ampersand,
    // FIXME: painted has no blank, but letterboard does.
}

impl From<Character> for char {
    fn from(value: Character) -> Self {
        match value {
            Character::A => 'A',
            Character::B => 'B',
            Character::C => 'C',
            Character::D => 'D',
            Character::E => 'E',
            Character::F => 'F',
            Character::G => 'G',
            Character::H => 'H',
            Character::I => 'I',
            Character::J => 'J',
            Character::K => 'K',
            Character::L => 'L',
            Character::M => 'M',
            Character::N => 'N',
            Character::O => 'O',
            Character::P => 'P',
            Character::Q => 'Q',
            Character::R => 'R',
            Character::S => 'S',
            Character::T => 'T',
            Character::U => 'U',
            Character::V => 'V',
            Character::W => 'W',
            Character::X => 'X',
            Character::Y => 'Y',
            Character::Z => 'Z',
            Character::DigL => '←',
            Character::DigR => '→',
            Character::DigU => '↑',
            Character::DigD => '↓',
            Character::Hash => '#',
            Character::At => '@',
            Character::Zero => '0',
            Character::One => '1',
            Character::Two => '2',
            Character::Three => '3',
            Character::Four => '4',
            Character::Five => '5',
            Character::Six => '6',
            Character::Seven => '7',
            Character::Eight => '8',
            Character::Nine => '9',
            Character::Dot => '.',
            Character::Colon => ':',
            Character::Slash => '/',
            Character::LParen => '(',
            Character::RParen => ')',
            Character::Ampersand => '&',
        }
    }
}

impl TryFrom<char> for Character {
    type Error = DecodeError;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value.to_ascii_uppercase() {
            'A' => Ok(Character::A),
            'B' => Ok(Character::B),
            'C' => Ok(Character::C),
            'D' => Ok(Character::D),
            'E' => Ok(Character::E),
            'F' => Ok(Character::F),
            'G' => Ok(Character::G),
            'H' => Ok(Character::H),
            'I' => Ok(Character::I),
            'J' => Ok(Character::J),
            'K' => Ok(Character::K),
            'L' => Ok(Character::L),
            'M' => Ok(Character::M),
            'N' => Ok(Character::N),
            'O' => Ok(Character::O),
            'P' => Ok(Character::P),
            'Q' => Ok(Character::Q),
            'R' => Ok(Character::R),
            'S' => Ok(Character::S),
            'T' => Ok(Character::T),
            'U' => Ok(Character::U),
            'V' => Ok(Character::V),
            'W' => Ok(Character::W),
            'X' => Ok(Character::X),
            'Y' => Ok(Character::Y),
            'Z' => Ok(Character::Z),
            '←' => Ok(Character::DigL),
            '→' => Ok(Character::DigR),
            '↑' => Ok(Character::DigU),
            '↓' => Ok(Character::DigD),
            '#' => Ok(Character::Hash),
            '@' => Ok(Character::At),
            '0' => Ok(Character::Zero),
            '1' => Ok(Character::One),
            '2' => Ok(Character::Two),
            '3' => Ok(Character::Three),
            '4' => Ok(Character::Four),
            '5' => Ok(Character::Five),
            '6' => Ok(Character::Six),
            '7' => Ok(Character::Seven),
            '8' => Ok(Character::Eight),
            '9' => Ok(Character::Nine),
            '.' => Ok(Character::Dot),
            ':' => Ok(Character::Colon),
            '/' => Ok(Character::Slash),
            '(' => Ok(Character::LParen),
            ')' => Ok(Character::RParen),
            '&' => Ok(Character::Ampersand),
            found => Err(DecodeError::BadMagic {
                found: Box::new(found),
            }),
        }
    }
}

impl TryFrom<u8> for Character {
    type Error = DecodeError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match (value & 0x7e) >> 1 {
            0 => Ok(Character::A),
            1 => Ok(Character::B),
            2 => Ok(Character::C),
            3 => Ok(Character::D),
            4 => Ok(Character::E),
            5 => Ok(Character::F),
            6 => Ok(Character::G),
            7 => Ok(Character::H),
            8 => Ok(Character::I),
            9 => Ok(Character::J),
            10 => Ok(Character::K),
            11 => Ok(Character::L),
            12 => Ok(Character::M),
            13 => Ok(Character::N),
            14 => Ok(Character::O),
            15 => Ok(Character::P),
            16 => Ok(Character::Q),
            17 => Ok(Character::R),
            18 => Ok(Character::S),
            19 => Ok(Character::T),
            20 => Ok(Character::U),
            21 => Ok(Character::V),
            22 => Ok(Character::W),
            23 => Ok(Character::X),
            24 => Ok(Character::Y),
            25 => Ok(Character::Z),
            26 => Ok(Character::DigL),
            27 => Ok(Character::DigR),
            28 => Ok(Character::DigU),
            29 => Ok(Character::DigD),
            30 => Ok(Character::Hash),
            31 => Ok(Character::At),
            32 => Ok(Character::Zero),
            33 => Ok(Character::One),
            34 => Ok(Character::Two),
            35 => Ok(Character::Three),
            36 => Ok(Character::Four),
            37 => Ok(Character::Five),
            38 => Ok(Character::Six),
            39 => Ok(Character::Seven),
            40 => Ok(Character::Eight),
            41 => Ok(Character::Nine),
            42 => Ok(Character::Dot),
            43 => Ok(Character::Colon),
            44 => Ok(Character::Slash),
            45 => Ok(Character::LParen),
            46 => Ok(Character::RParen),
            47 => Ok(Character::Ampersand),
            found => Err(DecodeError::NoVariantMatch {
                found: found as u64,
            }),
        }
    }
}

/// Painted Letters
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Letters {
    /// Colour
    pub colour: PaintColour,
    /// Character
    pub character: Character,
    /// Heading / Direction
    pub heading: Direction,
    /// Floating
    pub floating: bool,
}

impl Letters {
    /// Create painted letters from a string
    pub fn from_str(
        text: &str,
        colour: PaintColour,
        heading: Direction,
    ) -> Result<Vec<Letters>, DecodeError> {
        text.chars()
            .filter(|ch| *ch != ' ')
            .map(|ch| {
                let character = Character::try_from(ch)?;
                Ok(Letters {
                    colour,
                    character,
                    heading,
                    floating: false,
                })
            })
            .collect()
    }
}

impl ObjectVariant for Letters {
    fn to_wire(&self) -> Result<ObjectWire, crate::EncodeError> {
        let mut flags = 0;
        flags |= (self.character as u8) << 1;
        flags |= self.colour as u8 & 0x01;
        if self.floating {
            flags |= 0x80;
        }
        Ok(ObjectWire {
            flags,
            heading: self.heading.to_objectinfo_heading(),
        })
    }

    fn from_wire(wire: ObjectWire) -> Result<Self, crate::DecodeError> {
        let colour = PaintColour::from(wire.flags);
        let character = Character::try_from(wire.flags)?;
        let floating = wire.floating();
        Ok(Self {
            colour,
            character,
            heading: Direction::from_objectinfo_heading(wire.heading),
            floating,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[repr(u8)]
#[allow(missing_docs)]
#[non_exhaustive]
/// Painted Arrows
pub enum Arrow {
    #[default]
    Left = 0,
    Right,
    StraightL,
    StraightR,
    CurveL,
    CurveR,
    StraightOn,
}

impl TryFrom<u8> for Arrow {
    type Error = DecodeError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match (value & 0x07e) >> 1 {
            0 => Ok(Self::Left),
            1 => Ok(Self::Right),
            2 => Ok(Self::StraightL),
            3 => Ok(Self::StraightR),
            4 => Ok(Self::CurveL),
            5 => Ok(Self::CurveR),
            6 => Ok(Self::StraightOn),
            found => Err(DecodeError::NoVariantMatch {
                found: found as u64,
            }),
        }
    }
}

/// Painted Arrows
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Arrows {
    /// Colour
    pub colour: PaintColour,
    /// Arrow
    pub arrow: Arrow,
    /// Heading / Direction
    pub heading: Direction,
    /// Floating
    pub floating: bool,
}

impl ObjectVariant for Arrows {
    fn to_wire(&self) -> Result<ObjectWire, crate::EncodeError> {
        let mut flags = 0;
        flags |= (self.arrow as u8) << 1;
        flags |= self.colour as u8 & 0x01;
        if self.floating {
            flags |= 0x80;
        }
        Ok(ObjectWire {
            flags,
            heading: self.heading.to_objectinfo_heading(),
        })
    }

    fn from_wire(wire: ObjectWire) -> Result<Self, crate::DecodeError> {
        let colour = PaintColour::from(wire.flags);
        let arrow = Arrow::try_from(wire.flags)?;
        let floating = wire.floating();
        Ok(Self {
            colour,
            arrow,
            heading: Direction::from_objectinfo_heading(wire.heading),
            floating,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arrows_round_trip() {
        let original = Arrows::default();
        let wire = original.to_wire().expect("to_wire failed");
        let decoded = Arrows::from_wire(wire).expect("from_wire failed");
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_letters_round_trip() {
        let original = Letters::default();
        let wire = original.to_wire().expect("to_wire failed");
        let decoded = Letters::from_wire(wire).expect("from_wire failed");
        assert_eq!(original, decoded);
    }
}
