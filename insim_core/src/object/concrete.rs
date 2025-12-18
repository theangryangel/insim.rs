//! Concrete. Welcome to the jungle.
//! We have intentionally used enums here rather than primitives or newtypes to allow
//! LSPs/editors to very clearly indicate to users of the library what is valid and what is not.
//! Whilst this makes it feel slightly awkward, it is an intentional productivity boost.

use super::{ObjectVariant, ObjectWire};
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

/// Concrete Slab
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ConcreteSlab {
    /// Slab data
    pub slab: Slab,
    /// Heading / Direction
    pub heading: Direction,
}

impl ObjectVariant for ConcreteSlab {
    fn to_wire(&self) -> Result<ObjectWire, crate::EncodeError> {
        let mut flags = 0;
        flags |= self.slab.width as u8 & 0x03;
        flags |= (self.slab.length as u8 & 0x03) << 2;
        flags |= (self.slab.pitch as u8 & 0x0f) << 4;
        Ok(ObjectWire {
            index: 172,
            flags,
            heading: self.heading.to_objectinfo_heading(),
        })
    }

    fn from_wire(wire: ObjectWire) -> Result<Self, DecodeError> {
        let width = WidthLength::try_from(wire.flags & 0x03)?;
        let length = WidthLength::try_from((wire.flags & 0x0c) >> 2)?;
        let pitch = Pitch::try_from((wire.flags & 0xf0) >> 4)?;
        Ok(Self {
            slab: Slab {
                width,
                length,
                pitch,
            },
            heading: Direction::from_objectinfo_heading(wire.heading),
        })
    }
}

/// Concrete Ramp
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ConcreteRamp {
    /// Ramp data
    pub ramp: Ramp,
    /// Heading / Direction
    pub heading: Direction,
}

impl ObjectVariant for ConcreteRamp {
    fn to_wire(&self) -> Result<ObjectWire, crate::EncodeError> {
        let mut flags = 0;
        flags |= self.ramp.width as u8 & 0x03;
        flags |= (self.ramp.length as u8 & 0x03) << 2;
        flags |= (self.ramp.height as u8 & 0x0f) << 4;
        Ok(ObjectWire {
            index: 173,
            flags,
            heading: self.heading.to_objectinfo_heading(),
        })
    }

    fn from_wire(wire: ObjectWire) -> Result<Self, DecodeError> {
        let width = WidthLength::try_from(wire.flags & 0x03)?;
        let length = WidthLength::try_from((wire.flags & 0x0c) >> 2)?;
        let height = Height::try_from((wire.flags & 0xf0) >> 4)?;
        Ok(Self {
            ramp: Ramp {
                width,
                length,
                height,
            },
            heading: Direction::from_objectinfo_heading(wire.heading),
        })
    }
}

/// Concrete Wall
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ConcreteWall {
    /// Wall data
    pub wall: Wall,
    /// Heading / Direction
    pub heading: Direction,
}

impl ObjectVariant for ConcreteWall {
    fn to_wire(&self) -> Result<ObjectWire, crate::EncodeError> {
        let mut flags = 0;
        flags |= self.wall.colour as u8 & 0x03;
        flags |= (self.wall.length as u8 & 0x03) << 2;
        flags |= (self.wall.height as u8 & 0x0f) << 4;
        Ok(ObjectWire {
            index: 174,
            flags,
            heading: self.heading.to_objectinfo_heading(),
        })
    }

    fn from_wire(wire: ObjectWire) -> Result<Self, DecodeError> {
        let colour = Colour::try_from(wire.flags & 0x03)?;
        let length = WidthLength::try_from((wire.flags & 0x0c) >> 2)?;
        let height = Height::try_from((wire.flags & 0xf0) >> 4)?;
        Ok(Self {
            wall: Wall {
                colour,
                length,
                height,
            },
            heading: Direction::from_objectinfo_heading(wire.heading),
        })
    }
}

/// Concrete Pillar
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ConcretePillar {
    /// Pillar data
    pub pillar: Pillar,
    /// Heading / Direction
    pub heading: Direction,
}

impl ObjectVariant for ConcretePillar {
    fn to_wire(&self) -> Result<ObjectWire, crate::EncodeError> {
        let mut flags = 0;
        flags |= self.pillar.x as u8 & 0x03;
        flags |= (self.pillar.y as u8 & 0x03) << 2;
        flags |= (self.pillar.height as u8 & 0x0f) << 4;
        Ok(ObjectWire {
            index: 175,
            flags,
            heading: self.heading.to_objectinfo_heading(),
        })
    }

    fn from_wire(wire: ObjectWire) -> Result<Self, DecodeError> {
        let x = Size::try_from(wire.flags & 0x03)?;
        let y = Size::try_from((wire.flags & 0x0c) >> 2)?;
        let height = Height::try_from((wire.flags & 0xf0) >> 4)?;
        Ok(Self {
            pillar: Pillar { x, y, height },
            heading: Direction::from_objectinfo_heading(wire.heading),
        })
    }
}

/// Concrete Slab Wall
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ConcreteSlabWall {
    /// Slab Wall data
    pub slab_wall: SlabWall,
    /// Heading / Direction
    pub heading: Direction,
}

impl ObjectVariant for ConcreteSlabWall {
    fn to_wire(&self) -> Result<ObjectWire, crate::EncodeError> {
        let mut flags = 0;
        flags |= self.slab_wall.colour as u8 & 0x03;
        flags |= (self.slab_wall.length as u8 & 0x03) << 2;
        flags |= (self.slab_wall.pitch as u8 & 0x0f) << 4;
        Ok(ObjectWire {
            index: 176,
            flags,
            heading: self.heading.to_objectinfo_heading(),
        })
    }

    fn from_wire(wire: ObjectWire) -> Result<Self, DecodeError> {
        let colour = Colour::try_from(wire.flags & 0x03)?;
        let length = WidthLength::try_from((wire.flags & 0x0c) >> 2)?;
        let pitch = Pitch::try_from((wire.flags & 0xf0) >> 4)?;
        Ok(Self {
            slab_wall: SlabWall {
                colour,
                length,
                pitch,
            },
            heading: Direction::from_objectinfo_heading(wire.heading),
        })
    }
}

/// Concrete Ramp Wall
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ConcreteRampWall {
    /// Ramp Wall data
    pub ramp_wall: Wall,
    /// Heading / Direction
    pub heading: Direction,
}

impl ObjectVariant for ConcreteRampWall {
    fn to_wire(&self) -> Result<ObjectWire, crate::EncodeError> {
        let mut flags = 0;
        flags |= self.ramp_wall.colour as u8 & 0x03;
        flags |= (self.ramp_wall.length as u8 & 0x03) << 2;
        flags |= (self.ramp_wall.height as u8 & 0x0f) << 4;
        Ok(ObjectWire {
            index: 177,
            flags,
            heading: self.heading.to_objectinfo_heading(),
        })
    }

    fn from_wire(wire: ObjectWire) -> Result<Self, DecodeError> {
        let colour = Colour::try_from(wire.flags & 0x03)?;
        let length = WidthLength::try_from((wire.flags & 0x0c) >> 2)?;
        let height = Height::try_from((wire.flags & 0xf0) >> 4)?;
        Ok(Self {
            ramp_wall: Wall {
                colour,
                length,
                height,
            },
            heading: Direction::from_objectinfo_heading(wire.heading),
        })
    }
}

/// Concrete Short Slab Wall
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ConcreteShortSlabWall {
    /// Short Slab Wall data
    pub short_slab_wall: ShortSlabWall,
    /// Heading / Direction
    pub heading: Direction,
}

impl ObjectVariant for ConcreteShortSlabWall {
    fn to_wire(&self) -> Result<ObjectWire, crate::EncodeError> {
        let mut flags = 0;
        flags |= self.short_slab_wall.colour as u8 & 0x03;
        flags |= (self.short_slab_wall.y as u8 & 0x03) << 2;
        flags |= (self.short_slab_wall.pitch as u8 & 0x0f) << 4;
        Ok(ObjectWire {
            index: 178,
            flags,
            heading: self.heading.to_objectinfo_heading(),
        })
    }

    fn from_wire(wire: ObjectWire) -> Result<Self, DecodeError> {
        let colour = Colour::try_from(wire.flags & 0x03)?;
        let y = Size::try_from((wire.flags & 0x0c) >> 2)?;
        let pitch = Pitch::try_from((wire.flags & 0xf0) >> 4)?;
        Ok(Self {
            short_slab_wall: ShortSlabWall { colour, y, pitch },
            heading: Direction::from_objectinfo_heading(wire.heading),
        })
    }
}

/// Concrete Wedge
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ConcreteWedge {
    /// Wedge data
    pub wedge: Wedge,
    /// Heading / Direction
    pub heading: Direction,
}

impl ObjectVariant for ConcreteWedge {
    fn to_wire(&self) -> Result<ObjectWire, crate::EncodeError> {
        let mut flags = 0;
        flags |= self.wedge.colour as u8 & 0x03;
        flags |= (self.wedge.length as u8 & 0x03) << 2;
        flags |= (self.wedge.angle as u8 & 0x0f) << 4;
        Ok(ObjectWire {
            index: 179,
            flags,
            heading: self.heading.to_objectinfo_heading(),
        })
    }

    fn from_wire(wire: ObjectWire) -> Result<Self, DecodeError> {
        let colour = Colour::try_from(wire.flags & 0x03)?;
        let length = WidthLength::try_from((wire.flags & 0x0c) >> 2)?;
        let angle = Angle::try_from((wire.flags & 0xf0) >> 4)?;
        Ok(Self {
            wedge: Wedge {
                colour,
                length,
                angle,
            },
            heading: Direction::from_objectinfo_heading(wire.heading),
        })
    }
}
