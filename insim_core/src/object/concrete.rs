//! Concrete

use crate::{heading::Heading, object::{ObjectCoordinate, ObjectFlags}, DecodeError};

/// Represents Width and Length (2m, 4m, 8m, 16m)
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[repr(u8)]
#[allow(missing_docs)]
#[non_exhaustive]
pub enum ConcreteWidthLength {
    #[default]
    Two = 0,
    Four = 1,
    Eight = 2,
    Sixteen = 3,
}

impl TryFrom<u8> for ConcreteWidthLength {
    type Error = crate::DecodeError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(ConcreteWidthLength::Two),
            1 => Ok(ConcreteWidthLength::Four),
            2 => Ok(ConcreteWidthLength::Eight),
            3 => Ok(ConcreteWidthLength::Sixteen),
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
pub enum ConcreteColour {
    #[default]
    Grey = 0,
    Red = 1,
    Blue = 2,
    Yellow = 3,
}

impl TryFrom<u8> for ConcreteColour {
    type Error = crate::DecodeError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(ConcreteColour::Grey),
            1 => Ok(ConcreteColour::Red),
            2 => Ok(ConcreteColour::Blue),
            3 => Ok(ConcreteColour::Yellow),
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
pub enum ConcreteHeight {
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

impl TryFrom<u8> for ConcreteHeight {
    type Error = crate::DecodeError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(ConcreteHeight::M0_25),
            1 => Ok(ConcreteHeight::M0_50),
            2 => Ok(ConcreteHeight::M0_75),
            3 => Ok(ConcreteHeight::M1_00),
            4 => Ok(ConcreteHeight::M1_25),
            5 => Ok(ConcreteHeight::M1_50),
            6 => Ok(ConcreteHeight::M1_75),
            7 => Ok(ConcreteHeight::M2_00),
            8 => Ok(ConcreteHeight::M2_25),
            9 => Ok(ConcreteHeight::M2_50),
            10 => Ok(ConcreteHeight::M2_75),
            11 => Ok(ConcreteHeight::M3_00),
            12 => Ok(ConcreteHeight::M3_25),
            13 => Ok(ConcreteHeight::M3_50),
            14 => Ok(ConcreteHeight::M3_75),
            15 => Ok(ConcreteHeight::M4_00),
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
pub enum ConcretePitch {
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

impl TryFrom<u8> for ConcretePitch {
    type Error = crate::DecodeError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(ConcretePitch::Deg0),
            1 => Ok(ConcretePitch::Deg6),
            2 => Ok(ConcretePitch::Deg12),
            3 => Ok(ConcretePitch::Deg18),
            4 => Ok(ConcretePitch::Deg24),
            5 => Ok(ConcretePitch::Deg30),
            6 => Ok(ConcretePitch::Deg36),
            7 => Ok(ConcretePitch::Deg42),
            8 => Ok(ConcretePitch::Deg48),
            9 => Ok(ConcretePitch::Deg54),
            10 => Ok(ConcretePitch::Deg60),
            11 => Ok(ConcretePitch::Deg66),
            12 => Ok(ConcretePitch::Deg72),
            13 => Ok(ConcretePitch::Deg78),
            14 => Ok(ConcretePitch::Deg84),
            15 => Ok(ConcretePitch::Deg90),
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
pub enum ConcreteAngle {
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

impl TryFrom<u8> for ConcreteAngle {
    type Error = crate::DecodeError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(ConcreteAngle::Deg5_625),
            1 => Ok(ConcreteAngle::Deg11_25),
            2 => Ok(ConcreteAngle::Deg16_875),
            3 => Ok(ConcreteAngle::Deg22_5),
            4 => Ok(ConcreteAngle::Deg28_125),
            5 => Ok(ConcreteAngle::Deg33_75),
            6 => Ok(ConcreteAngle::Deg39_375),
            7 => Ok(ConcreteAngle::Deg45),
            8 => Ok(ConcreteAngle::Deg50_625),
            9 => Ok(ConcreteAngle::Deg56_25),
            10 => Ok(ConcreteAngle::Deg61_875),
            11 => Ok(ConcreteAngle::Deg67_5),
            12 => Ok(ConcreteAngle::Deg73_125),
            13 => Ok(ConcreteAngle::Deg78_75),
            14 => Ok(ConcreteAngle::Deg84_375),
            15 => Ok(ConcreteAngle::Deg90),
            _ => Err(crate::DecodeError::NoVariantMatch {
                found: value as u64,
            }),
        }
    }
}

/// Concrete Slab
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ConcreteSlab {
    /// Position
    pub xyz: ObjectCoordinate,
    /// Width
    pub width: ConcreteWidthLength,
    /// Length
    pub length: ConcreteWidthLength,
    /// Pitch
    pub pitch: ConcretePitch,
    /// Heading / Direction
    pub heading: Heading,
}

impl ConcreteSlab {
    pub(super) fn to_flags(&self) -> ObjectFlags {
        let mut flags = 0;
        flags |= self.width as u8 & 0x03;
        flags |= (self.length as u8 & 0x03) << 2;
        flags |= (self.pitch as u8 & 0x0f) << 4;
        ObjectFlags(flags)
    }

    pub(super) fn new(xyz: ObjectCoordinate, flags: ObjectFlags, heading: Heading) -> Result<Self, DecodeError> {
        let width = ConcreteWidthLength::try_from(flags.0 & 0x03)?;
        let length = ConcreteWidthLength::try_from((flags.0 & 0x0c) >> 2)?;
        let pitch = ConcretePitch::try_from((flags.0 & 0xf0) >> 4)?;
        Ok(Self {
            xyz,
            width,
            length,
            pitch,
            heading,
        })
    }
}

/// Concrete Ramp
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ConcreteRamp {
    /// Position
    pub xyz: ObjectCoordinate,
    /// Width
    pub width: ConcreteWidthLength,
    /// Length
    pub length: ConcreteWidthLength,
    /// Height
    pub height: ConcreteHeight,
    /// Heading / Direction
    pub heading: Heading,
}

impl ConcreteRamp {
    pub(super) fn to_flags(&self) -> ObjectFlags {
        let mut flags = 0;
        flags |= self.width as u8 & 0x03;
        flags |= (self.length as u8 & 0x03) << 2;
        flags |= (self.height as u8 & 0x0f) << 4;
        ObjectFlags(flags)
    }

    pub(super) fn new(xyz: ObjectCoordinate, wire: ObjectFlags, heading: Heading) -> Result<Self, DecodeError> {
        let width = ConcreteWidthLength::try_from(wire.0 & 0x03)?;
        let length = ConcreteWidthLength::try_from((wire.0 & 0x0c) >> 2)?;
        let height = ConcreteHeight::try_from((wire.0 & 0xf0) >> 4)?;
        Ok(Self {
            xyz,
            width,
            length,
            height,
            heading,
        })
    }
}

/// Concrete Wall
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ConcreteWall {
    /// Position
    pub xyz: ObjectCoordinate,
    /// Colour
    pub colour: ConcreteColour,
    /// Length
    pub length: ConcreteWidthLength,
    /// Height
    pub height: ConcreteHeight,
    /// Heading / Direction
    pub heading: Heading,
}

impl ConcreteWall {
    pub(super) fn to_wire(&self) -> ObjectFlags { 
        let mut flags = 0;
        flags |= self.colour as u8 & 0x03;
        flags |= (self.length as u8 & 0x03) << 2;
        flags |= (self.height as u8 & 0x0f) << 4;
        ObjectFlags(flags)
    }

    pub(super) fn new(xyz: ObjectCoordinate, wire: ObjectFlags, heading: Heading) -> Result<Self, DecodeError> {
        let colour = ConcreteColour::try_from(wire.0 & 0x03)?;
        let length = ConcreteWidthLength::try_from((wire.0 & 0x0c) >> 2)?;
        let height = ConcreteHeight::try_from((wire.0 & 0xf0) >> 4)?;
        Ok(Self {
            xyz,
            colour,
            length,
            height,
            heading,
        })
    }
}

/// Concrete Pillar
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ConcretePillar {
    /// Position
    pub xyz: ObjectCoordinate,
    /// SizeX
    pub x: Size,
    /// SizeY
    pub y: Size,
    /// Height
    pub height: ConcreteHeight,
    /// Heading / Direction
    pub heading: Heading,
}

impl ConcretePillar {
    pub(super) fn to_wire(&self) -> ObjectFlags {
        let mut flags = 0;
        flags |= self.x as u8 & 0x03;
        flags |= (self.y as u8 & 0x03) << 2;
        flags |= (self.height as u8 & 0x0f) << 4;
        ObjectFlags(flags)
    }

    pub(super) fn new(xyz: ObjectCoordinate, flags: ObjectFlags, heading: Heading) -> Result<Self, DecodeError> {
        let x = Size::try_from(flags.0 & 0x03)?;
        let y = Size::try_from((flags.0 & 0x0c) >> 2)?;
        let height = ConcreteHeight::try_from((flags.0 & 0xf0) >> 4)?;
        Ok(Self {
            xyz,
            x,
            y,
            height,
            heading,
        })
    }
}

/// Concrete Slab Wall
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ConcreteSlabWall {
    /// Position
    pub xyz: ObjectCoordinate,
    /// Colour
    pub colour: ConcreteColour,
    /// Length
    pub length: ConcreteWidthLength,
    /// Pitch
    pub pitch: ConcretePitch,
    /// Heading / Direction
    pub heading: Heading,
}

impl ConcreteSlabWall {
    pub(super) fn to_flags(&self) -> ObjectFlags {
        let mut flags = 0;
        flags |= self.colour as u8 & 0x03;
        flags |= (self.length as u8 & 0x03) << 2;
        flags |= (self.pitch as u8 & 0x0f) << 4;
        ObjectFlags(flags)
    }

    pub(super) fn new(xyz: ObjectCoordinate, wire: ObjectFlags, heading: Heading) -> Result<Self, DecodeError> {
        let colour = ConcreteColour::try_from(wire.0 & 0x03)?;
        let length = ConcreteWidthLength::try_from((wire.0 & 0x0c) >> 2)?;
        let pitch = ConcretePitch::try_from((wire.0 & 0xf0) >> 4)?;
        Ok(Self {
            xyz,
            colour,
            length,
            pitch,
            heading,
        })
    }
}

/// Concrete Ramp Wall
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ConcreteRampWall {
    /// Position
    pub xyz: ObjectCoordinate,
    /// Colour
    pub colour: ConcreteColour,
    /// Length
    pub length: ConcreteWidthLength,
    /// Height
    pub height: ConcreteHeight,
    /// Heading / Direction
    pub heading: Heading,
}

impl ConcreteRampWall {
    pub(super) fn to_wire(&self) -> ObjectFlags {
        let mut flags = 0;
        flags |= self.colour as u8 & 0x03;
        flags |= (self.length as u8 & 0x03) << 2;
        flags |= (self.height as u8 & 0x0f) << 4;
        ObjectFlags(flags)
    }

    pub(super) fn new(xyz: ObjectCoordinate, wire: ObjectFlags, heading: Heading) -> Result<Self, DecodeError> {
        let colour = ConcreteColour::try_from(wire.0 & 0x03)?;
        let length = ConcreteWidthLength::try_from((wire.0 & 0x0c) >> 2)?;
        let height = ConcreteHeight::try_from((wire.0 & 0xf0) >> 4)?;
        Ok(Self {
            xyz,
            colour,
            length,
            height,
            heading,
        })
    }
}

/// Concrete Short Slab Wall
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ConcreteShortSlabWall {
    /// Position
    pub xyz: ObjectCoordinate,
    /// Colour
    pub colour: ConcreteColour,
    /// Size Y
    pub y: Size,
    /// Pitch
    pub pitch: ConcretePitch,
    /// Heading / Direction
    pub heading: Heading,
}

impl ConcreteShortSlabWall {
    pub(super) fn to_flags(&self) -> ObjectFlags {
        let mut flags = 0;
        flags |= self.colour as u8 & 0x03;
        flags |= (self.y as u8 & 0x03) << 2;
        flags |= (self.pitch as u8 & 0x0f) << 4;
        ObjectFlags(flags)
    }

    pub(super) fn new(xyz: ObjectCoordinate, wire: ObjectFlags, heading: Heading) -> Result<Self, DecodeError> {
        let colour = ConcreteColour::try_from(wire.0 & 0x03)?;
        let y = Size::try_from((wire.0 & 0x0c) >> 2)?;
        let pitch = ConcretePitch::try_from((wire.0 & 0xf0) >> 4)?;
        Ok(Self {
            xyz,
            colour,
            y,
            pitch,
            heading,
        })
    }
}

/// Concrete Wedge
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ConcreteWedge {
    /// Position
    pub xyz: ObjectCoordinate,
    /// Colour
    pub colour: ConcreteColour,
    /// Length
    pub length: ConcreteWidthLength,
    /// Angle
    pub angle: ConcreteAngle,
    /// Heading / Direction
    pub heading: Heading,
}

impl ConcreteWedge {
    pub(super) fn to_flags(&self) -> ObjectFlags {
        let mut flags = 0;
        flags |= self.colour as u8 & 0x03;
        flags |= (self.length as u8 & 0x03) << 2;
        flags |= (self.angle as u8 & 0x0f) << 4;
        ObjectFlags(flags)
    }

    pub(super) fn new(xyz: ObjectCoordinate, wire: ObjectFlags, heading: Heading) -> Result<Self, DecodeError> {
        let colour = ConcreteColour::try_from(wire.0 & 0x03)?;
        let length = ConcreteWidthLength::try_from((wire.0 & 0x0c) >> 2)?;
        let angle = ConcreteAngle::try_from((wire.0 & 0xf0) >> 4)?;
        Ok(Self {
            xyz,
            colour,
            length,
            angle,
            heading,
        })
    }
}
