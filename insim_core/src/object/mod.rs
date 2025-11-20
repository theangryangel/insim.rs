//! Objects are used in both insim and lyt files

pub mod axo;
pub mod chalk;
pub mod concrete;
pub mod control;
pub mod tyre;

use crate::{Decode, DecodeError, Encode, EncodeError, object::control::InsimCircle};

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Object Position
pub struct ObjectPosition {
    /// X
    pub x: i16,
    /// Y
    pub y: i16,
    /// Z
    pub z: u8,
}

trait ObjectCodec: Sized {
    /// Encode this Object, returning (flags, heading)
    fn encode(&self) -> Result<(&ObjectPosition, u8, u8), EncodeError>;
    /// Bytes into an Object
    fn decode(xyz: ObjectPosition, flags: u8, heading: u8) -> Result<Self, DecodeError>;
}

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Standard Object values
pub struct Object {
    /// Object XYZ position
    pub xyz: ObjectPosition,
    /// Heading
    pub heading: u8,
    /// Floating
    pub floating: bool,
}

impl ObjectCodec for Object {
    fn encode(&self) -> Result<(&ObjectPosition, u8, u8), EncodeError> {
        let mut flags = 0;
        if self.floating {
            flags |= 0x80;
        }
        Ok((&self.xyz, flags, self.heading))
    }

    fn decode(xyz: ObjectPosition, flags: u8, heading: u8) -> Result<Self, DecodeError> {
        let floating = flags & 0x80 != 0;
        Ok(Self {
            xyz,
            heading,
            floating,
        })
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// ObjectInfo - used within insim and lyt files to describe user placeable objects
#[non_exhaustive]
pub enum ObjectInfo {
    #[default]
    /// Unknown
    Unknown,

    /// Control - Start position
    Start(control::Point),
    /// Control - Finish line
    Finish(control::Gate),
    /// Control - Checkpoint 1
    Checkpoint1(control::Gate),
    /// Control - Checkpoint 2
    Checkpoint2(control::Gate),
    /// Control - Checkpoint 3
    Checkpoint3(control::Gate),

    /// AXO_CHALK_LINE
    ChalkLine(chalk::Chalk),
    /// AXO_CHALK_LINE2
    ChalkLine2(chalk::Chalk),
    /// AXO_CHALK_AHEAD
    ChalkAhead(chalk::Chalk),
    /// AXO_CHALK_AHEAD2
    ChalkAhead2(chalk::Chalk),
    /// AXO_CHALK_LEFT
    ChalkLeft(chalk::Chalk),
    /// AXO_CHALK_LEFT2
    ChalkLeft2(chalk::Chalk),
    /// AXO_CHALK_LEFT3
    ChalkLeft3(chalk::Chalk),
    /// AXO_CHALK_RIGHT
    ChalkRight(chalk::Chalk),
    /// AXO_CHALK_RIGHT2
    ChalkRight2(chalk::Chalk),
    /// AXO_CHALK_RIGHT3
    ChalkRight3(chalk::Chalk),

    /// AXO_CONE_RED
    ConeRed(Object),
    /// AXO_CONE_RED2
    ConeRed2(Object),
    /// AXO_CONE_RED3
    ConeRed3(Object),
    /// AXO_CONE_BLUE
    ConeBlue(Object),
    /// AXO_CONE_BLUE2
    ConeBlue2(Object),
    /// AXO_CONE_GREEN
    ConeGreen(Object),
    /// AXO_CONE_GREEN2
    ConeGreen2(Object),
    /// AXO_CONE_ORANGE
    ConeOrange(Object),
    /// AXO_CONE_WHITE
    ConeWhite(Object),
    /// AXO_CONE_YELLOW
    ConeYellow(Object),
    /// AXO_CONE_YELLOW2
    ConeYellow2(Object),

    /// AXO_CONE_PTR_RED
    ConePtrRed(Object),
    /// AXO_CONE_PTR_BLUE
    ConePtrBlue(Object),
    /// AXO_CONE_PTR_GREEN
    ConePtrGreen(Object),
    /// AXO_CONE_PTR_YELLOW
    ConePtrYellow(Object),

    /// AXO_TYRE_SINGLE
    TyreSingle(tyre::TyreStack),
    /// AXO_TYRE_STACK2
    TyreStack2(tyre::TyreStack),
    /// AXO_TYRE_STACK3
    TyreStack3(tyre::TyreStack),
    /// AXO_TYRE_STACK4
    TyreStack4(tyre::TyreStack),
    /// AXO_TYRE_SINGLE_BIG
    TyreSingleBig(tyre::TyreStack),
    /// AXO_TYRE_STACK2_BIG
    TyreStack2Big(tyre::TyreStack),
    /// AXO_TYRE_STACK3_BIG
    TyreStack3Big(tyre::TyreStack),
    /// AXO_TYRE_STACK4_BIG
    TyreStack4Big(tyre::TyreStack),

    /// AXO_MARKER_CURVE_L
    MarkerCurveL(Object),
    /// AXO_MARKER_CURVE_R
    MarkerCurveR(Object),
    /// AXO_MARKER_L
    MarkerL(Object),
    /// AXO_MARKER_R
    MarkerR(Object),
    /// AXO_MARKER_HARD_L
    MarkerHardL(Object),
    /// AXO_MARKER_HARD_R
    MarkerHardR(Object),
    /// AXO_MARKER_L_R
    MarkerLR(Object),
    /// AXO_MARKER_R_L
    MarkerRL(Object),
    /// AXO_MARKER_S_L
    MarkerSL(Object),
    /// AXO_MARKER_S_R
    MarkerSR(Object),
    /// AXO_MARKER_S2_L
    MarkerS2L(Object),
    /// AXO_MARKER_S2_R
    MarkerS2R(Object),
    /// AXO_MARKER_U_L
    MarkerUL(Object),
    /// AXO_MARKER_U_R
    MarkerUR(Object),

    /// AXO_DIST25
    Dist25(Object),
    /// AXO_DIST50
    Dist50(Object),
    /// AXO_DIST75
    Dist75(Object),
    /// AXO_DIST100
    Dist100(Object),
    /// AXO_DIST125
    Dist125(Object),
    /// AXO_DIST150
    Dist150(Object),
    /// AXO_DIST200
    Dist200(Object),
    /// AXO_DIST250
    Dist250(Object),
    /// AXO_ARMCO1
    Armco1(Object),
    /// AXO_ARMCO3
    Armco3(Object),
    /// AXO_ARMCO5
    Armco5(Object),
    /// AXO_BARRIER_LONG
    BarrierLong(Object),
    /// AXO_BARRIER_RED
    BarrierRed(Object),
    /// AXO_BARRIER_WHITE
    BarrierWhite(Object),
    /// AXO_BANNER1
    Banner1(Object),
    /// AXO_BANNER2
    Banner2(Object),
    /// AXO_RAMP1
    Ramp1(Object),
    /// AXO_RAMP2
    Ramp2(Object),
    /// AXO_SPEED_HUMP_10M
    SpeedHump10M(Object),
    /// AXO_SPEED_HUMP_6M
    SpeedHump6M(Object),
    /// AXO_POST_GREEN
    PostGreen(Object),
    /// AXO_POST_ORANGE
    PostOrange(Object),
    /// AXO_POST_RED
    PostRed(Object),
    /// AXO_POST_WHITE
    PostWhite(Object),
    /// AXO_BALE
    Bale(Object),
    /// AXO_RAILING
    Railing(Object),
    /// AXO_START_LIGHTS
    StartLights(Object),
    /// AXO_SIGN_KEEP_LEFT
    SignKeepLeft(Object),
    /// AXO_SIGN_KEEP_RIGHT
    SignKeepRight(Object),
    /// AXO_SIGN_SPEED_80
    SignSpeed80(Object),
    /// AXO_SIGN_SPEED_50
    SignSpeed50(Object),

    /// AXO_CONCRETE_SLAB
    ConcreteSlab(concrete::Slab),
    /// AXO_CONCRETE_RAMP
    ConcreteRamp(concrete::Ramp),
    /// AXO_CONCRETE_WALL
    ConcreteWall(concrete::Wall),
    /// AXO_CONCRETE_PILLAR
    ConcretePillar(concrete::Pillar),
    /// AXO_CONCRETE_SLAB_WALL
    ConcreteSlabWall(concrete::SlabWall),
    /// AXO_CONCRETE_RAMP_WALL
    ConcreteRampWall(concrete::RampWall),
    /// AXO_CONCRETE_SHORT_SLAB_WALL
    ConcreteShortSlabWall(concrete::ShortSlabWall),
    /// AXO_CONCRETE_WEDGE
    ConcreteWedge(concrete::Wedge),

    /// AXO_START_POSITION
    AxoStartPosition(axo::AutoX),
    /// AXO_PIT_START_POINT
    AxoPitStartPoint(axo::AutoX),
    /// AXO_PIT_STOP_BOX
    AxoPitStopBox(Object),

    /// Insim Finish Checkpoint
    InsimCheckpointFinish(control::InsimCheckpoint),
    /// Insim Checkpoint 1
    InsimCheckpoint1(control::InsimCheckpoint),
    /// Insim Checkpoint 2
    InsimCheckpoint2(control::InsimCheckpoint),
    /// Insim Checkpoint 3
    InsimCheckpoint3(control::InsimCheckpoint),

    /// Insim Circle
    InsimCircle(control::InsimCircle),
    /// Marshall Circle / Restricted Area
    MarshallCircle(control::MarshallCircle),
    /// Route Checker
    RouteCheck(control::RouteCheck),
}

impl Decode for ObjectInfo {
    fn decode(buf: &mut bytes::Bytes) -> Result<Self, DecodeError> {
        let x = i16::decode(buf)?;
        let y = i16::decode(buf)?;
        let z = u8::decode(buf)?;

        let xyz = ObjectPosition { x, y, z };

        let flags = u8::decode(buf)?;
        let index = u8::decode(buf)?;
        let heading = u8::decode(buf)?;

        let res = match index {
            0 => {
                let position_bits = flags & 0b11;
                let half_width = (flags >> 2) & 0b11111;
                let floating = flags & 0x80 != 0;
                match position_bits {
                    0b00 if half_width == 0 => Self::Start(control::Point {
                        xyz,
                        heading,
                        floating,
                    }),
                    0b00 if half_width != 0 => Self::Finish(control::Gate {
                        xyz,
                        half_width,
                        heading,
                        floating,
                    }),
                    0b01 => Self::Checkpoint1(control::Gate {
                        xyz,
                        half_width,
                        heading,
                        floating,
                    }),
                    0b10 => Self::Checkpoint2(control::Gate {
                        xyz,
                        half_width,
                        heading,
                        floating,
                    }),
                    0b11 => Self::Checkpoint3(control::Gate {
                        xyz,
                        half_width,
                        heading,
                        floating,
                    }),
                    _ => {
                        return Err(DecodeError::NoVariantMatch {
                            found: index as u64,
                        });
                    },
                }
            },
            4 => Self::ChalkLine(chalk::Chalk::decode(xyz, flags, heading)?),
            5 => Self::ChalkLine2(chalk::Chalk::decode(xyz, flags, heading)?),
            6 => Self::ChalkAhead(chalk::Chalk::decode(xyz, flags, heading)?),
            7 => Self::ChalkAhead2(chalk::Chalk::decode(xyz, flags, heading)?),
            8 => Self::ChalkLeft(chalk::Chalk::decode(xyz, flags, heading)?),
            9 => Self::ChalkLeft2(chalk::Chalk::decode(xyz, flags, heading)?),
            10 => Self::ChalkLeft3(chalk::Chalk::decode(xyz, flags, heading)?),
            11 => Self::ChalkRight(chalk::Chalk::decode(xyz, flags, heading)?),
            12 => Self::ChalkRight2(chalk::Chalk::decode(xyz, flags, heading)?),
            13 => Self::ChalkRight3(chalk::Chalk::decode(xyz, flags, heading)?),
            20 => Self::ConeRed(Object::decode(xyz, flags, heading)?),
            21 => Self::ConeRed2(Object::decode(xyz, flags, heading)?),
            22 => Self::ConeRed3(Object::decode(xyz, flags, heading)?),
            23 => Self::ConeBlue(Object::decode(xyz, flags, heading)?),
            24 => Self::ConeBlue2(Object::decode(xyz, flags, heading)?),
            25 => Self::ConeGreen(Object::decode(xyz, flags, heading)?),
            26 => Self::ConeGreen2(Object::decode(xyz, flags, heading)?),
            27 => Self::ConeOrange(Object::decode(xyz, flags, heading)?),
            28 => Self::ConeWhite(Object::decode(xyz, flags, heading)?),
            29 => Self::ConeYellow(Object::decode(xyz, flags, heading)?),
            30 => Self::ConeYellow2(Object::decode(xyz, flags, heading)?),
            40 => Self::ConePtrRed(Object::decode(xyz, flags, heading)?),
            41 => Self::ConePtrBlue(Object::decode(xyz, flags, heading)?),
            42 => Self::ConePtrGreen(Object::decode(xyz, flags, heading)?),
            43 => Self::ConePtrYellow(Object::decode(xyz, flags, heading)?),
            48 => Self::TyreSingle(tyre::TyreStack::decode(xyz, flags, heading)?),
            49 => Self::TyreStack2(tyre::TyreStack::decode(xyz, flags, heading)?),
            50 => Self::TyreStack3(tyre::TyreStack::decode(xyz, flags, heading)?),
            51 => Self::TyreStack4(tyre::TyreStack::decode(xyz, flags, heading)?),
            52 => Self::TyreSingleBig(tyre::TyreStack::decode(xyz, flags, heading)?),
            53 => Self::TyreStack2Big(tyre::TyreStack::decode(xyz, flags, heading)?),
            54 => Self::TyreStack3Big(tyre::TyreStack::decode(xyz, flags, heading)?),
            55 => Self::TyreStack4Big(tyre::TyreStack::decode(xyz, flags, heading)?),
            64 => Self::MarkerCurveL(Object::decode(xyz, flags, heading)?),
            65 => Self::MarkerCurveR(Object::decode(xyz, flags, heading)?),
            66 => Self::MarkerL(Object::decode(xyz, flags, heading)?),
            67 => Self::MarkerR(Object::decode(xyz, flags, heading)?),
            68 => Self::MarkerHardL(Object::decode(xyz, flags, heading)?),
            69 => Self::MarkerHardR(Object::decode(xyz, flags, heading)?),
            70 => Self::MarkerLR(Object::decode(xyz, flags, heading)?),
            71 => Self::MarkerRL(Object::decode(xyz, flags, heading)?),
            72 => Self::MarkerSL(Object::decode(xyz, flags, heading)?),
            73 => Self::MarkerSR(Object::decode(xyz, flags, heading)?),
            74 => Self::MarkerS2L(Object::decode(xyz, flags, heading)?),
            75 => Self::MarkerS2R(Object::decode(xyz, flags, heading)?),
            76 => Self::MarkerUL(Object::decode(xyz, flags, heading)?),
            77 => Self::MarkerUR(Object::decode(xyz, flags, heading)?),
            84 => Self::Dist25(Object::decode(xyz, flags, heading)?),
            85 => Self::Dist50(Object::decode(xyz, flags, heading)?),
            86 => Self::Dist75(Object::decode(xyz, flags, heading)?),
            87 => Self::Dist100(Object::decode(xyz, flags, heading)?),
            88 => Self::Dist125(Object::decode(xyz, flags, heading)?),
            89 => Self::Dist150(Object::decode(xyz, flags, heading)?),
            90 => Self::Dist200(Object::decode(xyz, flags, heading)?),
            91 => Self::Dist250(Object::decode(xyz, flags, heading)?),
            96 => Self::Armco1(Object::decode(xyz, flags, heading)?),
            97 => Self::Armco3(Object::decode(xyz, flags, heading)?),
            98 => Self::Armco5(Object::decode(xyz, flags, heading)?),
            104 => Self::BarrierLong(Object::decode(xyz, flags, heading)?),
            105 => Self::BarrierRed(Object::decode(xyz, flags, heading)?),
            106 => Self::BarrierWhite(Object::decode(xyz, flags, heading)?),
            112 => Self::Banner1(Object::decode(xyz, flags, heading)?),
            113 => Self::Banner2(Object::decode(xyz, flags, heading)?),
            120 => Self::Ramp1(Object::decode(xyz, flags, heading)?),
            121 => Self::Ramp2(Object::decode(xyz, flags, heading)?),
            128 => Self::SpeedHump10M(Object::decode(xyz, flags, heading)?),
            129 => Self::SpeedHump6M(Object::decode(xyz, flags, heading)?),
            136 => Self::PostGreen(Object::decode(xyz, flags, heading)?),
            137 => Self::PostOrange(Object::decode(xyz, flags, heading)?),
            138 => Self::PostRed(Object::decode(xyz, flags, heading)?),
            139 => Self::PostWhite(Object::decode(xyz, flags, heading)?),
            144 => Self::Bale(Object::decode(xyz, flags, heading)?),
            148 => Self::Railing(Object::decode(xyz, flags, heading)?),
            149 => Self::StartLights(Object::decode(xyz, flags, heading)?),
            160 => Self::SignKeepLeft(Object::decode(xyz, flags, heading)?),
            161 => Self::SignKeepRight(Object::decode(xyz, flags, heading)?),
            168 => Self::SignSpeed80(Object::decode(xyz, flags, heading)?),
            169 => Self::SignSpeed50(Object::decode(xyz, flags, heading)?),
            172 => Self::ConcreteSlab(concrete::Slab::decode(xyz, flags, heading)?),
            173 => Self::ConcreteRamp(concrete::Ramp::decode(xyz, flags, heading)?),
            174 => Self::ConcreteWall(concrete::Wall::decode(xyz, flags, heading)?),
            175 => Self::ConcretePillar(concrete::Pillar::decode(xyz, flags, heading)?),
            176 => Self::ConcreteSlabWall(concrete::SlabWall::decode(xyz, flags, heading)?),
            177 => Self::ConcreteRampWall(concrete::RampWall::decode(xyz, flags, heading)?),
            178 => {
                Self::ConcreteShortSlabWall(concrete::ShortSlabWall::decode(xyz, flags, heading)?)
            },
            179 => Self::ConcreteWedge(concrete::Wedge::decode(xyz, flags, heading)?),
            184 => Self::AxoStartPosition(axo::AutoX::decode(xyz, flags, heading)?),
            185 => Self::AxoPitStartPoint(axo::AutoX::decode(xyz, flags, heading)?),
            186 => Self::AxoPitStopBox(Object::decode(xyz, flags, heading)?),

            252 => {
                let floating = flags & 0x80 != 0;

                match flags & 0x03 {
                    0 => Self::InsimCheckpointFinish(control::InsimCheckpoint {
                        xyz,
                        heading,
                        floating,
                    }),
                    1 => Self::InsimCheckpoint1(control::InsimCheckpoint {
                        xyz,
                        heading,
                        floating,
                    }),
                    2 => Self::InsimCheckpoint2(control::InsimCheckpoint {
                        xyz,
                        heading,
                        floating,
                    }),
                    3 => Self::InsimCheckpoint3(control::InsimCheckpoint {
                        xyz,
                        heading,
                        floating,
                    }),
                    _ => {
                        return Err(DecodeError::NoVariantMatch {
                            found: flags as u64,
                        });
                    },
                }
            },
            253 => Self::InsimCircle(InsimCircle {
                xyz,
                flags,
                index: heading,
            }),
            254 => Self::MarshallCircle(control::MarshallCircle::decode(xyz, flags, heading)?),
            255 => Self::RouteCheck(control::RouteCheck::decode(xyz, flags, heading)?),
            _ => {
                return Err(DecodeError::NoVariantMatch {
                    found: index as u64,
                });
            },
        };

        Ok(res)
    }
}

impl Encode for ObjectInfo {
    fn encode(&self, buf: &mut bytes::BytesMut) -> Result<(), EncodeError> {
        let (index, xyz, flags, heading): (u8, &ObjectPosition, u8, u8) = match self {
            Self::Start(ctrl) => {
                let mut flags = 0;
                if ctrl.floating {
                    flags |= 0x80;
                }
                (0, &ctrl.xyz, flags, ctrl.heading)
            },
            Self::Finish(ctrl) => {
                if ctrl.half_width == 0 {
                    return Err(EncodeError::TooSmall);
                }
                let mut flags = ctrl.half_width << 2;
                if ctrl.floating {
                    flags |= 0x80;
                }
                (0, &ctrl.xyz, flags, ctrl.heading)
            },
            Self::Checkpoint1(ctrl) => {
                let mut flags = (ctrl.half_width << 2) | 0b01;
                if ctrl.floating {
                    flags |= 0x80;
                }
                (0, &ctrl.xyz, flags, ctrl.heading)
            },
            Self::Checkpoint2(ctrl) => {
                let mut flags = (ctrl.half_width << 2) | 0b10;
                if ctrl.floating {
                    flags |= 0x80;
                }
                (0, &ctrl.xyz, flags, ctrl.heading)
            },
            Self::Checkpoint3(ctrl) => {
                let mut flags = (ctrl.half_width << 2) | 0b11;
                if ctrl.floating {
                    flags |= 0x80;
                }
                (0, &ctrl.xyz, flags, ctrl.heading)
            },
            Self::ChalkLine(chalk) => {
                let (xyz, flags, heading) = chalk.encode()?;
                (4, xyz, flags, heading)
            },
            Self::ChalkLine2(chalk) => {
                let (xyz, flags, heading) = chalk.encode()?;
                (5, xyz, flags, heading)
            },
            Self::ChalkAhead(chalk) => {
                let (xyz, flags, heading) = chalk.encode()?;
                (6, xyz, flags, heading)
            },
            Self::ChalkAhead2(chalk) => {
                let (xyz, flags, heading) = chalk.encode()?;
                (7, xyz, flags, heading)
            },
            Self::ChalkLeft(chalk) => {
                let (xyz, flags, heading) = chalk.encode()?;
                (8, xyz, flags, heading)
            },
            Self::ChalkLeft2(chalk) => {
                let (xyz, flags, heading) = chalk.encode()?;
                (9, xyz, flags, heading)
            },
            Self::ChalkLeft3(chalk) => {
                let (xyz, flags, heading) = chalk.encode()?;
                (10, xyz, flags, heading)
            },
            Self::ChalkRight(chalk) => {
                let (xyz, flags, heading) = chalk.encode()?;
                (11, xyz, flags, heading)
            },
            Self::ChalkRight2(chalk) => {
                let (xyz, flags, heading) = chalk.encode()?;
                (12, xyz, flags, heading)
            },
            Self::ChalkRight3(chalk) => {
                let (xyz, flags, heading) = chalk.encode()?;
                (13, xyz, flags, heading)
            },
            Self::ConeRed(obj) => {
                let (xyz, flags, heading) = obj.encode()?;
                (20, xyz, flags, heading)
            },
            Self::ConeRed2(obj) => {
                let (xyz, flags, heading) = obj.encode()?;
                (21, xyz, flags, heading)
            },
            Self::ConeRed3(obj) => {
                let (xyz, flags, heading) = obj.encode()?;
                (22, xyz, flags, heading)
            },
            Self::ConeBlue(obj) => {
                let (xyz, flags, heading) = obj.encode()?;
                (23, xyz, flags, heading)
            },
            Self::ConeBlue2(obj) => {
                let (xyz, flags, heading) = obj.encode()?;
                (24, xyz, flags, heading)
            },
            Self::ConeGreen(obj) => {
                let (xyz, flags, heading) = obj.encode()?;
                (25, xyz, flags, heading)
            },
            Self::ConeGreen2(obj) => {
                let (xyz, flags, heading) = obj.encode()?;
                (26, xyz, flags, heading)
            },
            Self::ConeOrange(obj) => {
                let (xyz, flags, heading) = obj.encode()?;
                (27, xyz, flags, heading)
            },
            Self::ConeWhite(obj) => {
                let (xyz, flags, heading) = obj.encode()?;
                (28, xyz, flags, heading)
            },
            Self::ConeYellow(obj) => {
                let (xyz, flags, heading) = obj.encode()?;
                (29, xyz, flags, heading)
            },
            Self::ConeYellow2(obj) => {
                let (xyz, flags, heading) = obj.encode()?;
                (30, xyz, flags, heading)
            },
            Self::ConePtrRed(obj) => {
                let (xyz, flags, heading) = obj.encode()?;
                (40, xyz, flags, heading)
            },
            Self::ConePtrBlue(obj) => {
                let (xyz, flags, heading) = obj.encode()?;
                (41, xyz, flags, heading)
            },
            Self::ConePtrGreen(obj) => {
                let (xyz, flags, heading) = obj.encode()?;
                (42, xyz, flags, heading)
            },
            Self::ConePtrYellow(obj) => {
                let (xyz, flags, heading) = obj.encode()?;
                (43, xyz, flags, heading)
            },
            Self::TyreSingle(tyre) => {
                let (xyz, flags, heading) = tyre.encode()?;
                (48, xyz, flags, heading)
            },
            Self::TyreStack2(tyre) => {
                let (xyz, flags, heading) = tyre.encode()?;
                (49, xyz, flags, heading)
            },
            Self::TyreStack3(tyre) => {
                let (xyz, flags, heading) = tyre.encode()?;
                (50, xyz, flags, heading)
            },
            Self::TyreStack4(tyre) => {
                let (xyz, flags, heading) = tyre.encode()?;
                (51, xyz, flags, heading)
            },
            Self::TyreSingleBig(tyre) => {
                let (xyz, flags, heading) = tyre.encode()?;
                (52, xyz, flags, heading)
            },
            Self::TyreStack2Big(tyre) => {
                let (xyz, flags, heading) = tyre.encode()?;
                (53, xyz, flags, heading)
            },
            Self::TyreStack3Big(tyre) => {
                let (xyz, flags, heading) = tyre.encode()?;
                (54, xyz, flags, heading)
            },
            Self::TyreStack4Big(tyre) => {
                let (xyz, flags, heading) = tyre.encode()?;
                (55, xyz, flags, heading)
            },
            Self::MarkerCurveL(obj) => {
                let (xyz, flags, heading) = obj.encode()?;
                (64, xyz, flags, heading)
            },
            Self::MarkerCurveR(obj) => {
                let (xyz, flags, heading) = obj.encode()?;
                (65, xyz, flags, heading)
            },
            Self::MarkerL(obj) => {
                let (xyz, flags, heading) = obj.encode()?;
                (66, xyz, flags, heading)
            },
            Self::MarkerR(obj) => {
                let (xyz, flags, heading) = obj.encode()?;
                (67, xyz, flags, heading)
            },
            Self::MarkerHardL(obj) => {
                let (xyz, flags, heading) = obj.encode()?;
                (68, xyz, flags, heading)
            },
            Self::MarkerHardR(obj) => {
                let (xyz, flags, heading) = obj.encode()?;
                (69, xyz, flags, heading)
            },
            Self::MarkerLR(obj) => {
                let (xyz, flags, heading) = obj.encode()?;
                (70, xyz, flags, heading)
            },
            Self::MarkerRL(obj) => {
                let (xyz, flags, heading) = obj.encode()?;
                (71, xyz, flags, heading)
            },
            Self::MarkerSL(obj) => {
                let (xyz, flags, heading) = obj.encode()?;
                (72, xyz, flags, heading)
            },
            Self::MarkerSR(obj) => {
                let (xyz, flags, heading) = obj.encode()?;
                (73, xyz, flags, heading)
            },
            Self::MarkerS2L(obj) => {
                let (xyz, flags, heading) = obj.encode()?;
                (74, xyz, flags, heading)
            },
            Self::MarkerS2R(obj) => {
                let (xyz, flags, heading) = obj.encode()?;
                (75, xyz, flags, heading)
            },
            Self::MarkerUL(obj) => {
                let (xyz, flags, heading) = obj.encode()?;
                (76, xyz, flags, heading)
            },
            Self::MarkerUR(obj) => {
                let (xyz, flags, heading) = obj.encode()?;
                (77, xyz, flags, heading)
            },
            Self::Dist25(obj) => {
                let (xyz, flags, heading) = obj.encode()?;
                (84, xyz, flags, heading)
            },
            Self::Dist50(obj) => {
                let (xyz, flags, heading) = obj.encode()?;
                (85, xyz, flags, heading)
            },
            Self::Dist75(obj) => {
                let (xyz, flags, heading) = obj.encode()?;
                (86, xyz, flags, heading)
            },
            Self::Dist100(obj) => {
                let (xyz, flags, heading) = obj.encode()?;
                (87, xyz, flags, heading)
            },
            Self::Dist125(obj) => {
                let (xyz, flags, heading) = obj.encode()?;
                (88, xyz, flags, heading)
            },
            Self::Dist150(obj) => {
                let (xyz, flags, heading) = obj.encode()?;
                (89, xyz, flags, heading)
            },
            Self::Dist200(obj) => {
                let (xyz, flags, heading) = obj.encode()?;
                (90, xyz, flags, heading)
            },
            Self::Dist250(obj) => {
                let (xyz, flags, heading) = obj.encode()?;
                (91, xyz, flags, heading)
            },
            Self::Armco1(obj) => {
                let (xyz, flags, heading) = obj.encode()?;
                (96, xyz, flags, heading)
            },
            Self::Armco3(obj) => {
                let (xyz, flags, heading) = obj.encode()?;
                (97, xyz, flags, heading)
            },
            Self::Armco5(obj) => {
                let (xyz, flags, heading) = obj.encode()?;
                (98, xyz, flags, heading)
            },
            Self::BarrierLong(obj) => {
                let (xyz, flags, heading) = obj.encode()?;
                (104, xyz, flags, heading)
            },
            Self::BarrierRed(obj) => {
                let (xyz, flags, heading) = obj.encode()?;
                (105, xyz, flags, heading)
            },
            Self::BarrierWhite(obj) => {
                let (xyz, flags, heading) = obj.encode()?;
                (106, xyz, flags, heading)
            },
            Self::Banner1(obj) => {
                let (xyz, flags, heading) = obj.encode()?;
                (112, xyz, flags, heading)
            },
            Self::Banner2(obj) => {
                let (xyz, flags, heading) = obj.encode()?;
                (113, xyz, flags, heading)
            },
            Self::Ramp1(obj) => {
                let (xyz, flags, heading) = obj.encode()?;
                (120, xyz, flags, heading)
            },
            Self::Ramp2(obj) => {
                let (xyz, flags, heading) = obj.encode()?;
                (121, xyz, flags, heading)
            },
            Self::SpeedHump10M(obj) => {
                let (xyz, flags, heading) = obj.encode()?;
                (128, xyz, flags, heading)
            },
            Self::SpeedHump6M(obj) => {
                let (xyz, flags, heading) = obj.encode()?;
                (129, xyz, flags, heading)
            },
            Self::PostGreen(obj) => {
                let (xyz, flags, heading) = obj.encode()?;
                (136, xyz, flags, heading)
            },
            Self::PostOrange(obj) => {
                let (xyz, flags, heading) = obj.encode()?;
                (137, xyz, flags, heading)
            },
            Self::PostRed(obj) => {
                let (xyz, flags, heading) = obj.encode()?;
                (138, xyz, flags, heading)
            },
            Self::PostWhite(obj) => {
                let (xyz, flags, heading) = obj.encode()?;
                (139, xyz, flags, heading)
            },
            Self::Bale(obj) => {
                let (xyz, flags, heading) = obj.encode()?;
                (144, xyz, flags, heading)
            },
            Self::Railing(obj) => {
                let (xyz, flags, heading) = obj.encode()?;
                (148, xyz, flags, heading)
            },
            Self::StartLights(obj) => {
                let (xyz, flags, heading) = obj.encode()?;
                (149, xyz, flags, heading)
            },
            Self::SignKeepLeft(obj) => {
                let (xyz, flags, heading) = obj.encode()?;
                (160, xyz, flags, heading)
            },
            Self::SignKeepRight(obj) => {
                let (xyz, flags, heading) = obj.encode()?;
                (161, xyz, flags, heading)
            },
            Self::SignSpeed80(obj) => {
                let (xyz, flags, heading) = obj.encode()?;
                (168, xyz, flags, heading)
            },
            Self::SignSpeed50(obj) => {
                let (xyz, flags, heading) = obj.encode()?;
                (169, xyz, flags, heading)
            },
            Self::ConcreteSlab(slab) => {
                let (xyz, flags, heading) = slab.encode()?;
                (172, xyz, flags, heading)
            },
            Self::ConcreteRamp(ramp) => {
                let (xyz, flags, heading) = ramp.encode()?;
                (173, xyz, flags, heading)
            },
            Self::ConcreteWall(wall) => {
                let (xyz, flags, heading) = wall.encode()?;
                (174, xyz, flags, heading)
            },
            Self::ConcretePillar(pillar) => {
                let (xyz, flags, heading) = pillar.encode()?;
                (175, xyz, flags, heading)
            },
            Self::ConcreteSlabWall(slab_wall) => {
                let (xyz, flags, heading) = slab_wall.encode()?;
                (176, xyz, flags, heading)
            },
            Self::ConcreteRampWall(ramp_wall) => {
                let (xyz, flags, heading) = ramp_wall.encode()?;
                (177, xyz, flags, heading)
            },
            Self::ConcreteShortSlabWall(short_slab_wall) => {
                let (xyz, flags, heading) = short_slab_wall.encode()?;
                (178, xyz, flags, heading)
            },
            Self::ConcreteWedge(wedge) => {
                let (xyz, flags, heading) = wedge.encode()?;
                (179, xyz, flags, heading)
            },
            Self::AxoStartPosition(axo) => {
                let (xyz, flags, heading) = axo.encode()?;
                (184, xyz, flags, heading)
            },
            Self::AxoPitStartPoint(axo) => {
                let (xyz, flags, heading) = axo.encode()?;
                (185, xyz, flags, heading)
            },
            Self::AxoPitStopBox(obj) => {
                let (xyz, flags, heading) = obj.encode()?;
                (186, xyz, flags, heading)
            },
            Self::InsimCheckpointFinish(cp) => (252, &cp.xyz, 0b00, cp.heading),
            Self::InsimCheckpoint1(cp) => (252, &cp.xyz, 0b01, cp.heading),
            Self::InsimCheckpoint2(cp) => (252, &cp.xyz, 0b10, cp.heading),
            Self::InsimCheckpoint3(cp) => (252, &cp.xyz, 0b11, cp.heading),
            Self::InsimCircle(circle) => (253, &circle.xyz, circle.flags, circle.index),
            Self::MarshallCircle(marshall) => {
                let (xyz, flags, heading) = marshall.encode()?;
                (254, xyz, flags, heading)
            },
            Self::RouteCheck(route) => {
                let (xyz, flags, heading) = route.encode()?;
                (255, xyz, flags, heading)
            },
            Self::Unknown => {
                return Err(EncodeError::NoVariantMatch { found: 0 });
            },
        };
        xyz.x.encode(buf)?;
        xyz.y.encode(buf)?;
        xyz.z.encode(buf)?;
        flags.encode(buf)?;
        index.encode(buf)?;
        heading.encode(buf)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use bytes::BytesMut;

    use super::*;
    use crate::object::{concrete::WidthLength, control::Marshall};

    #[test]
    fn test_start_roundtrip() {
        let original = ObjectInfo::Start(control::Point {
            xyz: ObjectPosition {
                x: 100,
                y: 200,
                z: 5,
            },
            heading: 128,
            floating: true,
        });

        let mut buf = BytesMut::new();
        original.encode(&mut buf).unwrap();

        let mut bytes = buf.freeze();
        let decoded = ObjectInfo::decode(&mut bytes).unwrap();

        match (original, decoded) {
            (ObjectInfo::Start(orig), ObjectInfo::Start(dec)) => {
                assert_eq!(orig.xyz.x, dec.xyz.x);
                assert_eq!(orig.xyz.y, dec.xyz.y);
                assert_eq!(orig.xyz.z, dec.xyz.z);
                assert_eq!(orig.heading, dec.heading);
            },
            _ => panic!("Variant mismatch"),
        }
    }

    #[test]
    fn test_finish_roundtrip() {
        let original = ObjectInfo::Finish(control::Gate {
            xyz: ObjectPosition {
                x: -500,
                y: 1000,
                z: 10,
            },
            half_width: 15,
            heading: 64,
            floating: false,
        });

        let mut buf = BytesMut::new();
        original.encode(&mut buf).unwrap();

        let mut bytes = buf.freeze();
        let decoded = ObjectInfo::decode(&mut bytes).unwrap();

        match (original, decoded) {
            (ObjectInfo::Finish(orig), ObjectInfo::Finish(dec)) => {
                assert_eq!(orig.xyz.x, dec.xyz.x);
                assert_eq!(orig.xyz.y, dec.xyz.y);
                assert_eq!(orig.xyz.z, dec.xyz.z);
                assert_eq!(orig.half_width, dec.half_width);
                assert_eq!(orig.heading, dec.heading);
            },
            _ => panic!("Variant mismatch"),
        }
    }

    #[test]
    fn test_checkpoint1_roundtrip() {
        let original = ObjectInfo::Checkpoint1(control::Gate {
            xyz: ObjectPosition {
                x: 250,
                y: -300,
                z: 8,
            },
            half_width: 20,
            heading: 192,
            floating: false,
        });

        let mut buf = BytesMut::new();
        original.encode(&mut buf).unwrap();

        let mut bytes = buf.freeze();
        let decoded = ObjectInfo::decode(&mut bytes).unwrap();

        match (original, decoded) {
            (ObjectInfo::Checkpoint1(orig), ObjectInfo::Checkpoint1(dec)) => {
                assert_eq!(orig.xyz.x, dec.xyz.x);
                assert_eq!(orig.xyz.y, dec.xyz.y);
                assert_eq!(orig.xyz.z, dec.xyz.z);
                assert_eq!(orig.half_width, dec.half_width);
                assert_eq!(orig.heading, dec.heading);
            },
            _ => panic!("Variant mismatch"),
        }
    }

    #[test]
    fn test_checkpoint2_roundtrip() {
        let original = ObjectInfo::Checkpoint2(control::Gate {
            xyz: ObjectPosition { x: 0, y: 0, z: 0 },
            half_width: 10,
            heading: 0,
            floating: false,
        });

        let mut buf = BytesMut::new();
        original.encode(&mut buf).unwrap();

        let mut bytes = buf.freeze();
        let decoded = ObjectInfo::decode(&mut bytes).unwrap();

        match (original, decoded) {
            (ObjectInfo::Checkpoint2(orig), ObjectInfo::Checkpoint2(dec)) => {
                assert_eq!(orig.xyz.x, dec.xyz.x);
                assert_eq!(orig.xyz.y, dec.xyz.y);
                assert_eq!(orig.xyz.z, dec.xyz.z);
                assert_eq!(orig.half_width, dec.half_width);
                assert_eq!(orig.heading, dec.heading);
            },
            _ => panic!("Variant mismatch"),
        }
    }

    #[test]
    fn test_checkpoint3_roundtrip() {
        let original = ObjectInfo::Checkpoint3(control::Gate {
            xyz: ObjectPosition {
                x: 750,
                y: 850,
                z: 12,
            },
            half_width: 25,
            heading: 255,
            floating: false,
        });

        let mut buf = BytesMut::new();
        original.encode(&mut buf).unwrap();

        let mut bytes = buf.freeze();
        let decoded = ObjectInfo::decode(&mut bytes).unwrap();

        match (original, decoded) {
            (ObjectInfo::Checkpoint3(orig), ObjectInfo::Checkpoint3(dec)) => {
                assert_eq!(orig.xyz.x, dec.xyz.x);
                assert_eq!(orig.xyz.y, dec.xyz.y);
                assert_eq!(orig.xyz.z, dec.xyz.z);
                assert_eq!(orig.half_width, dec.half_width);
                assert_eq!(orig.heading, dec.heading);
            },
            _ => panic!("Variant mismatch"),
        }
    }

    #[test]
    fn test_concrete_slab_roundtrip() {
        let slab = concrete::Slab {
            xyz: ObjectPosition {
                x: 300,
                y: 400,
                z: 2,
            },
            width: WidthLength::Two,
            length: WidthLength::Four,
            pitch: concrete::Pitch::Deg0,
            heading: 90,
        };
        let original = ObjectInfo::ConcreteSlab(slab);

        let mut buf = BytesMut::new();
        original.encode(&mut buf).unwrap();

        let mut bytes = buf.freeze();
        let decoded = ObjectInfo::decode(&mut bytes).unwrap();

        match (original, decoded) {
            (ObjectInfo::ConcreteSlab(orig), ObjectInfo::ConcreteSlab(dec)) => {
                assert_eq!(orig.xyz.x, dec.xyz.x);
                assert_eq!(orig.xyz.y, dec.xyz.y);
                assert_eq!(orig.xyz.z, dec.xyz.z);
                assert_eq!(orig.width, dec.width);
                assert_eq!(orig.length, dec.length);
                assert_eq!(orig.pitch, dec.pitch);
                assert_eq!(orig.heading, dec.heading);
            },
            _ => panic!("Variant mismatch"),
        }
    }

    #[test]
    fn test_marshall_circle_roundtrip() {
        let marshall = control::MarshallCircle {
            xyz: ObjectPosition {
                x: -100,
                y: 600,
                z: 15,
            },
            marshall: Marshall::Standing,
            radius: 2,
            heading: 180,
            floating: false,
        };
        let original = ObjectInfo::MarshallCircle(marshall);

        let mut buf = BytesMut::new();
        original.encode(&mut buf).unwrap();

        let mut bytes = buf.freeze();
        let decoded = ObjectInfo::decode(&mut bytes).unwrap();

        match (original, decoded) {
            (ObjectInfo::MarshallCircle(orig), ObjectInfo::MarshallCircle(dec)) => {
                assert_eq!(orig.xyz.x, dec.xyz.x);
                assert_eq!(orig.xyz.y, dec.xyz.y);
                assert_eq!(orig.xyz.z, dec.xyz.z);
                assert_eq!(orig.marshall, dec.marshall);
                assert_eq!(orig.heading, dec.heading);
            },
            _ => panic!("Variant mismatch"),
        }
    }

    #[test]
    fn test_route_check_roundtrip() {
        let route = control::RouteCheck {
            xyz: ObjectPosition {
                x: 1000,
                y: -500,
                z: 20,
            },
            radius: 6,
            floating: false,
            heading: 45,
        };
        let original = ObjectInfo::RouteCheck(route);

        let mut buf = BytesMut::new();
        original.encode(&mut buf).unwrap();

        let mut bytes = buf.freeze();
        let decoded = ObjectInfo::decode(&mut bytes).unwrap();

        match (original, decoded) {
            (ObjectInfo::RouteCheck(orig), ObjectInfo::RouteCheck(dec)) => {
                assert_eq!(orig.xyz.x, dec.xyz.x);
                assert_eq!(orig.xyz.y, dec.xyz.y);
                assert_eq!(orig.xyz.z, dec.xyz.z);
                assert_eq!(orig.radius, dec.radius);
                assert_eq!(orig.floating, dec.floating);
                assert_eq!(orig.heading, dec.heading);
            },
            _ => panic!("Variant mismatch"),
        }
    }

    #[test]
    fn test_cone_red_roundtrip() {
        let obj = Object {
            xyz: ObjectPosition {
                x: 150,
                y: 250,
                z: 3,
            },
            heading: 32,
            floating: true,
        };
        let original = ObjectInfo::ConeRed(obj);

        let mut buf = BytesMut::new();
        original.encode(&mut buf).unwrap();

        let mut bytes = buf.freeze();
        let decoded = ObjectInfo::decode(&mut bytes).unwrap();

        match (original, decoded) {
            (ObjectInfo::ConeRed(orig), ObjectInfo::ConeRed(dec)) => {
                assert_eq!(orig.xyz.x, dec.xyz.x);
                assert_eq!(orig.xyz.y, dec.xyz.y);
                assert_eq!(orig.xyz.z, dec.xyz.z);
                assert_eq!(orig.floating, dec.floating);
                assert_eq!(orig.heading, dec.heading);
            },
            _ => panic!("Variant mismatch"),
        }
    }

    #[test]
    fn test_barrier_long_roundtrip() {
        let obj = Object {
            xyz: ObjectPosition {
                x: -250,
                y: -750,
                z: 7,
            },
            floating: false,
            heading: 220,
        };
        let original = ObjectInfo::BarrierLong(obj);

        let mut buf = BytesMut::new();
        original.encode(&mut buf).unwrap();

        let mut bytes = buf.freeze();
        let decoded = ObjectInfo::decode(&mut bytes).unwrap();

        match (original, decoded) {
            (ObjectInfo::BarrierLong(orig), ObjectInfo::BarrierLong(dec)) => {
                assert_eq!(orig.xyz.x, dec.xyz.x);
                assert_eq!(orig.xyz.y, dec.xyz.y);
                assert_eq!(orig.xyz.z, dec.xyz.z);
                assert_eq!(orig.floating, dec.floating);
                assert_eq!(orig.heading, dec.heading);
            },
            _ => panic!("Variant mismatch"),
        }
    }

    #[test]
    fn test_insim_checkpoint_finish_roundtrip() {
        let obj = control::InsimCheckpoint {
            xyz: ObjectPosition {
                x: -250,
                y: -750,
                z: 7,
            },
            floating: false,
            heading: 220,
        };
        let original = ObjectInfo::InsimCheckpointFinish(obj);

        let mut buf = BytesMut::new();
        original.encode(&mut buf).unwrap();

        let mut bytes = buf.freeze();
        let decoded = ObjectInfo::decode(&mut bytes).unwrap();

        match (&original, &decoded) {
            (ObjectInfo::InsimCheckpointFinish(orig), ObjectInfo::InsimCheckpointFinish(dec)) => {
                assert_eq!(orig.xyz.x, dec.xyz.x);
                assert_eq!(orig.xyz.y, dec.xyz.y);
                assert_eq!(orig.xyz.z, dec.xyz.z);
                assert_eq!(orig.floating, dec.floating);
                assert_eq!(orig.heading, dec.heading);
            },
            _ => panic!("Variant mismatch {:?}, {:?}", original, decoded),
        }
    }

    #[test]
    fn test_insim_checkpoint1_finish_roundtrip() {
        let obj = control::InsimCheckpoint {
            xyz: ObjectPosition {
                x: -250,
                y: -750,
                z: 7,
            },
            floating: false,
            heading: 220,
        };
        let original = ObjectInfo::InsimCheckpoint1(obj);

        let mut buf = BytesMut::new();
        original.encode(&mut buf).unwrap();

        let mut bytes = buf.freeze();
        let decoded = ObjectInfo::decode(&mut bytes).unwrap();

        match (&original, &decoded) {
            (ObjectInfo::InsimCheckpoint1(orig), ObjectInfo::InsimCheckpoint1(dec)) => {
                assert_eq!(orig.xyz.x, dec.xyz.x);
                assert_eq!(orig.xyz.y, dec.xyz.y);
                assert_eq!(orig.xyz.z, dec.xyz.z);
                assert_eq!(orig.floating, dec.floating);
                assert_eq!(orig.heading, dec.heading);
            },
            _ => panic!("Variant mismatch {:?}, {:?}", original, decoded),
        }
    }

    #[test]
    fn test_insim_checkpoint2_finish_roundtrip() {
        let obj = control::InsimCheckpoint {
            xyz: ObjectPosition {
                x: -250,
                y: -750,
                z: 7,
            },
            floating: false,
            heading: 220,
        };
        let original = ObjectInfo::InsimCheckpoint2(obj);

        let mut buf = BytesMut::new();
        original.encode(&mut buf).unwrap();

        let mut bytes = buf.freeze();
        let decoded = ObjectInfo::decode(&mut bytes).unwrap();

        match (&original, &decoded) {
            (ObjectInfo::InsimCheckpoint2(orig), ObjectInfo::InsimCheckpoint2(dec)) => {
                assert_eq!(orig.xyz.x, dec.xyz.x);
                assert_eq!(orig.xyz.y, dec.xyz.y);
                assert_eq!(orig.xyz.z, dec.xyz.z);
                assert_eq!(orig.floating, dec.floating);
                assert_eq!(orig.heading, dec.heading);
            },
            _ => panic!("Variant mismatch {:?}, {:?}", original, decoded),
        }
    }

    #[test]
    fn test_insim_checkpoint3_finish_roundtrip() {
        let obj = control::InsimCheckpoint {
            xyz: ObjectPosition {
                x: -250,
                y: -750,
                z: 7,
            },
            floating: false,
            heading: 220,
        };
        let original = ObjectInfo::InsimCheckpoint3(obj);

        let mut buf = BytesMut::new();
        original.encode(&mut buf).unwrap();

        let mut bytes = buf.freeze();
        let decoded = ObjectInfo::decode(&mut bytes).unwrap();

        match (&original, &decoded) {
            (ObjectInfo::InsimCheckpoint3(orig), ObjectInfo::InsimCheckpoint3(dec)) => {
                assert_eq!(orig.xyz.x, dec.xyz.x);
                assert_eq!(orig.xyz.y, dec.xyz.y);
                assert_eq!(orig.xyz.z, dec.xyz.z);
                assert_eq!(orig.floating, dec.floating);
                assert_eq!(orig.heading, dec.heading);
            },
            _ => panic!("Variant mismatch {:?}, {:?}", original, decoded),
        }
    }
}
