//! Concrete. Welcome to the jungle.
//! We have intentionally used enums here rather than primitives or newtypes to allow
//! LSPs/editors to very clearly indicate to users of the library what is valid and what is not.
//! Whilst this makes it feel slightly awkward, it is an intentional productivity boost.

use super::ObjectVariant;
use crate::{DecodeError, direction::Direction};

/// Represents Width and Length (2m, 4m, 8m, 16m)
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[repr(u8)]
#[allow(missing_docs)]
#[non_exhaustive]
pub enum WidthLength {
    #[default]
    Two = 0,
    Four = 1,
    Eight = 2,
    Sixteen = 3,
}

impl TryFrom<u8> for WidthLength {
    type Error = crate::DecodeError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(WidthLength::Two),
            1 => Ok(WidthLength::Four),
            2 => Ok(WidthLength::Eight),
            3 => Ok(WidthLength::Sixteen),
            _ => Err(crate::DecodeError::NoVariantMatch {
                found: value as u64,
            }),
        }
    }
}
/// Represents Size X/Y (0.25x to 1.0x)
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[repr(u8)]
#[allow(missing_docs)]
#[non_exhaustive]
pub enum Size {
    #[default]
    Quarter = 0, // 0.25
    Half = 1,         // 0.50
    ThreeQuarter = 2, // 0.75
    Full = 3,         // 1.00
}

impl TryFrom<u8> for Size {
    type Error = crate::DecodeError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Size::Quarter),
            1 => Ok(Size::Half),
            2 => Ok(Size::ThreeQuarter),
            3 => Ok(Size::Full),
            _ => Err(crate::DecodeError::NoVariantMatch {
                found: value as u64,
            }),
        }
    }
}

/// Represents Colour options
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[repr(u8)]
#[allow(missing_docs)]
#[non_exhaustive]
pub enum Colour {
    #[default]
    Grey = 0,
    Red = 1,
    Blue = 2,
    Yellow = 3,
}

impl TryFrom<u8> for Colour {
    type Error = crate::DecodeError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Colour::Grey),
            1 => Ok(Colour::Red),
            2 => Ok(Colour::Blue),
            3 => Ok(Colour::Yellow),
            _ => Err(crate::DecodeError::NoVariantMatch {
                found: value as u64,
            }),
        }
    }
}

/// Concrete kind variants
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[non_exhaustive]
pub enum ConcreteKind {
    /// Slab (width, length, pitch)
    Slab(Slab),
    /// Ramp (width, length, height)
    Ramp(Ramp),
    /// Wall (colour, length, height)
    Wall(Wall),
    /// Pillar (size x, size y, height)
    Pillar(Pillar),
    /// Slab wall (colour, length, pitch)
    SlabWall(SlabWall),
    /// Ramp wall (colour, length, height)
    RampWall(RampWall),
    /// Short slab wall (colour, size y, pitch)
    ShortSlabWall(ShortSlabWall),
    /// Wedge (colour, length, angle)
    Wedge(Wedge),
}

impl ConcreteKind {
    /// Get index for this concrete kind
    pub fn index(&self) -> u8 {
        match self {
            ConcreteKind::Slab(_) => 172,
            ConcreteKind::Ramp(_) => 173,
            ConcreteKind::Wall(_) => 174,
            ConcreteKind::Pillar(_) => 175,
            ConcreteKind::SlabWall(_) => 176,
            ConcreteKind::RampWall(_) => 177,
            ConcreteKind::ShortSlabWall(_) => 178,
            ConcreteKind::Wedge(_) => 179,
        }
    }
}

/// Unified Concrete object
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Concrete {
    /// Kind of concrete object
    pub kind: ConcreteKind,
    /// Heading / Direction
    pub heading: Direction,
}

/// Represents Height in 0.25m steps (0.25m to 4.0m)
/// Using specific enum variants allows IDE autocomplete to guide the user.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[repr(u8)]
#[allow(missing_docs)]
#[non_exhaustive]
pub enum Height {
    #[default]
    M0_25 = 0,
    M0_50 = 1,
    M0_75 = 2,
    M1_00 = 3,
    M1_25 = 4,
    M1_50 = 5,
    M1_75 = 6,
    M2_00 = 7,
    M2_25 = 8,
    M2_50 = 9,
    M2_75 = 10,
    M3_00 = 11,
    M3_25 = 12,
    M3_50 = 13,
    M3_75 = 14,
    M4_00 = 15,
}

impl TryFrom<u8> for Height {
    type Error = crate::DecodeError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Height::M0_25),
            1 => Ok(Height::M0_50),
            2 => Ok(Height::M0_75),
            3 => Ok(Height::M1_00),
            4 => Ok(Height::M1_25),
            5 => Ok(Height::M1_50),
            6 => Ok(Height::M1_75),
            7 => Ok(Height::M2_00),
            8 => Ok(Height::M2_25),
            9 => Ok(Height::M2_50),
            10 => Ok(Height::M2_75),
            11 => Ok(Height::M3_00),
            12 => Ok(Height::M3_25),
            13 => Ok(Height::M3_50),
            14 => Ok(Height::M3_75),
            15 => Ok(Height::M4_00),
            _ => Err(crate::DecodeError::NoVariantMatch {
                found: value as u64,
            }),
        }
    }
}

/// Represents Pitch from 0 to 90 degrees in 6-degree steps.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[repr(u8)]
#[allow(missing_docs)]
#[non_exhaustive]
pub enum Pitch {
    #[default]
    Deg0 = 0,
    Deg6 = 1,
    Deg12 = 2,
    Deg18 = 3,
    Deg24 = 4,
    Deg30 = 5,
    Deg36 = 6,
    Deg42 = 7,
    Deg48 = 8,
    Deg54 = 9,
    Deg60 = 10,
    Deg66 = 11,
    Deg72 = 12,
    Deg78 = 13,
    Deg84 = 14,
    Deg90 = 15,
}

impl TryFrom<u8> for Pitch {
    type Error = crate::DecodeError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Pitch::Deg0),
            1 => Ok(Pitch::Deg6),
            2 => Ok(Pitch::Deg12),
            3 => Ok(Pitch::Deg18),
            4 => Ok(Pitch::Deg24),
            5 => Ok(Pitch::Deg30),
            6 => Ok(Pitch::Deg36),
            7 => Ok(Pitch::Deg42),
            8 => Ok(Pitch::Deg48),
            9 => Ok(Pitch::Deg54),
            10 => Ok(Pitch::Deg60),
            11 => Ok(Pitch::Deg66),
            12 => Ok(Pitch::Deg72),
            13 => Ok(Pitch::Deg78),
            14 => Ok(Pitch::Deg84),
            15 => Ok(Pitch::Deg90),
            _ => Err(crate::DecodeError::NoVariantMatch {
                found: value as u64,
            }),
        }
    }
}

/// Represents Angle from 5.625 to 90 degrees in 5.625 steps.
/// Naming simplifies the fractional decimals for readability.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[repr(u8)]
#[allow(missing_docs)]
#[non_exhaustive]
pub enum Angle {
    #[default]
    Deg5_625 = 0,
    Deg11_25 = 1,
    Deg16_875 = 2,
    Deg22_5 = 3,
    Deg28_125 = 4,
    Deg33_75 = 5,
    Deg39_375 = 6,
    Deg45 = 7,
    Deg50_625 = 8,
    Deg56_25 = 9,
    Deg61_875 = 10,
    Deg67_5 = 11,
    Deg73_125 = 12,
    Deg78_75 = 13,
    Deg84_375 = 14,
    Deg90 = 15,
}

impl TryFrom<u8> for Angle {
    type Error = crate::DecodeError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Angle::Deg5_625),
            1 => Ok(Angle::Deg11_25),
            2 => Ok(Angle::Deg16_875),
            3 => Ok(Angle::Deg22_5),
            4 => Ok(Angle::Deg28_125),
            5 => Ok(Angle::Deg33_75),
            6 => Ok(Angle::Deg39_375),
            7 => Ok(Angle::Deg45),
            8 => Ok(Angle::Deg50_625),
            9 => Ok(Angle::Deg56_25),
            10 => Ok(Angle::Deg61_875),
            11 => Ok(Angle::Deg67_5),
            12 => Ok(Angle::Deg73_125),
            13 => Ok(Angle::Deg78_75),
            14 => Ok(Angle::Deg84_375),
            15 => Ok(Angle::Deg90),
            _ => Err(crate::DecodeError::NoVariantMatch {
                found: value as u64,
            }),
        }
    }
}

/// Slab
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Slab {
    /// Width
    pub width: WidthLength,
    /// Length
    pub length: WidthLength,
    /// Pitch
    pub pitch: Pitch,
}

/// Ramp
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Ramp {
    /// Width
    pub width: WidthLength,
    /// Length
    pub length: WidthLength,
    /// Height
    pub height: Height,
}

/// Wall
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Wall {
    /// Colour
    pub colour: Colour,
    /// Length
    pub length: WidthLength,
    /// Height
    pub height: Height,
}

/// Ramp Wall
pub type RampWall = Wall;

/// Pillar
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Pillar {
    /// SizeX
    pub x: Size,
    /// SizeY
    pub y: Size,
    /// Height
    pub height: Height,
}

/// SlabWall
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct SlabWall {
    /// Colour
    pub colour: Colour,
    /// Length
    pub length: WidthLength,
    /// Pitch
    pub pitch: Pitch,
}

/// ShortSlabWall
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ShortSlabWall {
    /// Colour
    pub colour: Colour,
    /// Size Y
    pub y: Size,
    /// Pitch
    pub pitch: Pitch,
}

/// Wedge
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Wedge {
    /// Colour
    pub colour: Colour,
    /// Length
    pub length: WidthLength,
    /// Angle
    pub angle: Angle,
}

impl ObjectVariant for Concrete {
    fn encode(&self) -> Result<(u8, u8, u8), crate::EncodeError> {
        let index = self.kind.index();
        let mut flags = 0;

        match &self.kind {
            ConcreteKind::Slab(slab) => {
                flags |= slab.width as u8 & 0x03;
                flags |= (slab.length as u8 & 0x03) << 2;
                flags |= (slab.pitch as u8 & 0x0f) << 4;
            },
            ConcreteKind::Ramp(ramp) => {
                flags |= ramp.width as u8 & 0x03;
                flags |= (ramp.length as u8 & 0x03) << 2;
                flags |= (ramp.height as u8 & 0x0f) << 4;
            },
            ConcreteKind::Wall(wall) => {
                flags |= wall.colour as u8 & 0x03;
                flags |= (wall.length as u8 & 0x03) << 2;
                flags |= (wall.height as u8 & 0x0f) << 4;
            },
            ConcreteKind::Pillar(pillar) => {
                flags |= pillar.x as u8 & 0x03;
                flags |= (pillar.y as u8 & 0x03) << 2;
                flags |= (pillar.height as u8 & 0x0f) << 4;
            },
            ConcreteKind::SlabWall(slab_wall) => {
                flags |= slab_wall.colour as u8 & 0x03;
                flags |= (slab_wall.length as u8 & 0x03) << 2;
                flags |= (slab_wall.pitch as u8 & 0x0f) << 4;
            },
            ConcreteKind::RampWall(ramp_wall) => {
                flags |= ramp_wall.colour as u8 & 0x03;
                flags |= (ramp_wall.length as u8 & 0x03) << 2;
                flags |= (ramp_wall.height as u8 & 0x0f) << 4;
            },
            ConcreteKind::ShortSlabWall(short_slab_wall) => {
                flags |= short_slab_wall.colour as u8 & 0x03;
                flags |= (short_slab_wall.y as u8 & 0x03) << 2;
                flags |= (short_slab_wall.pitch as u8 & 0x0f) << 4;
            },
            ConcreteKind::Wedge(wedge) => {
                flags |= wedge.colour as u8 & 0x03;
                flags |= (wedge.length as u8 & 0x03) << 2;
                flags |= (wedge.angle as u8 & 0x0f) << 4;
            },
        }

        let heading = self.heading.to_objectinfo_heading();
        Ok((index, flags, heading))
    }

    fn decode(index: u8, flags: u8, heading: u8) -> Result<Self, DecodeError> {
        let kind = match index {
            172 => {
                let width = WidthLength::try_from(flags & 0x03)?;
                let length = WidthLength::try_from((flags & 0x0c) >> 2)?;
                let pitch = Pitch::try_from((flags & 0xf0) >> 4)?;
                ConcreteKind::Slab(Slab {
                    width,
                    length,
                    pitch,
                })
            },
            173 => {
                let width = WidthLength::try_from(flags & 0x03)?;
                let length = WidthLength::try_from((flags & 0x0c) >> 2)?;
                let height = Height::try_from((flags & 0xf0) >> 4)?;
                ConcreteKind::Ramp(Ramp {
                    width,
                    length,
                    height,
                })
            },
            174 => {
                let colour = Colour::try_from(flags & 0x03)?;
                let length = WidthLength::try_from((flags & 0x0c) >> 2)?;
                let height = Height::try_from((flags & 0xf0) >> 4)?;
                ConcreteKind::Wall(Wall {
                    colour,
                    length,
                    height,
                })
            },
            175 => {
                let x = Size::try_from(flags & 0x03)?;
                let y = Size::try_from((flags & 0x0c) >> 2)?;
                let height = Height::try_from((flags & 0xf0) >> 4)?;
                ConcreteKind::Pillar(Pillar { x, y, height })
            },
            176 => {
                let colour = Colour::try_from(flags & 0x03)?;
                let length = WidthLength::try_from((flags & 0x0c) >> 2)?;
                let pitch = Pitch::try_from((flags & 0xf0) >> 4)?;
                ConcreteKind::SlabWall(SlabWall {
                    colour,
                    length,
                    pitch,
                })
            },
            177 => {
                let colour = Colour::try_from(flags & 0x03)?;
                let length = WidthLength::try_from((flags & 0x0c) >> 2)?;
                let height = Height::try_from((flags & 0xf0) >> 4)?;
                ConcreteKind::RampWall(RampWall {
                    colour,
                    length,
                    height,
                })
            },
            178 => {
                let colour = Colour::try_from(flags & 0x03)?;
                let y = Size::try_from((flags & 0x0c) >> 2)?;
                let pitch = Pitch::try_from((flags & 0xf0) >> 4)?;
                ConcreteKind::ShortSlabWall(ShortSlabWall { colour, y, pitch })
            },
            179 => {
                let colour = Colour::try_from(flags & 0x03)?;
                let length = WidthLength::try_from((flags & 0x0c) >> 2)?;
                let angle = Angle::try_from((flags & 0xf0) >> 4)?;
                ConcreteKind::Wedge(Wedge {
                    colour,
                    length,
                    angle,
                })
            },
            _ => {
                return Err(DecodeError::NoVariantMatch {
                    found: index as u64,
                });
            },
        };

        Ok(Concrete {
            kind,
            heading: Direction::from_objectinfo_heading(heading),
        })
    }
}
