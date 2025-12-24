//! Objects are used in both insim and lyt files

pub mod armco;
pub mod bale;
pub mod banner;
pub mod barrier;
pub mod bin1;
pub mod bin2;
pub mod chalk;
pub mod chevron;
pub mod concrete;
pub mod cones;
pub mod control;
pub mod insim;
pub mod kerb;
pub mod letterboard_rb;
pub mod letterboard_wy;
pub mod marker;
pub mod marquee;
pub mod marshal;
pub mod painted;
pub mod pit;
pub mod pit_start_point;
pub mod post;
pub mod railing;
pub mod ramp;
pub mod sign_metal;
pub mod sign_speed;
pub mod speed_hump;
pub mod start_lights;
pub mod start_position;
pub mod tyres;
pub mod vehicle_ambulance;
pub mod vehicle_suv;
pub mod vehicle_truck;
pub mod vehicle_van;

use crate::{Decode, DecodeError, Encode, EncodeError};

/// Wire representation for object encoding/decoding
#[derive(Debug, Clone, Copy)]
pub(crate) struct ObjectWire {
    /// Flags byte (semantics depend on object type)
    pub flags: u8,
    /// Heading/data byte (semantics depend on object type)
    pub heading: u8,
}

impl ObjectWire {
    /// Check if the floating flag is set
    pub fn floating(&self) -> bool {
        self.flags & 0x80 != 0
    }

    /// Extract colour from flags (bits 0-2)
    pub fn colour(&self) -> u8 {
        self.flags & 0x07
    }

    /// Extract mapping from flags (bits 3-6)
    pub fn mapping(&self) -> u8 {
        (self.flags >> 3) & 0x0f
    }
}

trait ObjectVariant: Sized {
    /// Encode this Object to wire format (returns flags and heading only)
    fn to_wire(&self) -> Result<ObjectWire, EncodeError>;
    /// Decode Object from wire format
    fn from_wire(wire: ObjectWire) -> Result<Self, DecodeError>;
}

#[derive(Debug, Copy, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Layout Object Position
pub struct ObjectCoordinate {
    x: i16,
    y: i16,
    z: u8,
}

impl ObjectCoordinate {
    // Scale to metres for X and Y
    const SCALE: i16 = 16;

    /// X (in metres)
    pub fn x_metres(&self) -> f32 {
        self.x as f32 / Self::SCALE as f32
    }

    /// Y (in metres)
    pub fn y_metres(&self) -> f32 {
        self.y as f32 / Self::SCALE as f32
    }

    /// Z (in metres)
    pub fn z_metres(&self) -> f32 {
        self.z as f32 / 4.0
    }

    /// X, Y, Z (in metres)
    pub fn xyz_metres(&self) -> (f32, f32, f32) {
        (self.x_metres(), self.y_metres(), self.z_metres())
    }
}

#[cfg(feature = "glam")]
impl ObjectCoordinate {
    /// Convert to glam Vec3, where xyz are in raw
    pub fn to_ivec3(&self) -> glam::I16Vec3 {
        glam::I16Vec3 {
            x: self.x,
            y: self.y,
            z: self.z as i16,
        }
    }

    /// Convert from glam IVec3, where xyz are in raw
    pub fn from_ivec3(other: glam::I16Vec3) -> Self {
        Self {
            x: other.x,
            y: other.y,
            z: other.z as u8,
        }
    }

    /// Convert to glam DVec3, where xyz are in metres
    pub fn to_dvec3_metres(&self) -> glam::DVec3 {
        glam::DVec3 {
            x: (self.x as f64 / 16.0),
            y: (self.y as f64 / 16.0),
            z: (self.y as f64 / 4.0),
        }
    }

    /// Convert from glam DVec3, where xyz are in metres
    pub fn from_dvec3_metres(other: glam::DVec3) -> Self {
        Self {
            x: (other.x * 16.0).round() as i16,
            y: (other.y * 16.0).round() as i16,
            z: (other.z * 4.0).round() as u8,
        }
    }

    /// Convert to glam Vec3, where xyz are in metres
    pub fn to_vec3_metres(&self) -> glam::Vec3 {
        glam::Vec3 {
            x: (self.x as f32 / 16.0),
            y: (self.y as f32 / 16.0),
            z: (self.y as f32 / 4.0),
        }
    }

    /// Convert from glam Vec3, where xyz are in metres
    pub fn from_vec3_metres(other: glam::Vec3) -> Self {
        Self {
            x: (other.x * 16.0).round() as i16,
            y: (other.y * 16.0).round() as i16,
            z: (other.z * 4.0).round() as u8,
        }
    }
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Layout Object
pub struct ObjectInfo {
    /// Object xyz position
    pub xyz: ObjectCoordinate,
    /// Kind
    pub kind: ObjectKind,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[non_exhaustive]
/// Layout Object Kind
pub enum ObjectKind {
    /// Control - start, finish, checkpoints
    Control(control::Control),
    /// Marshal
    Marshal(marshal::Marshal),
    /// Insim Checkpoint
    InsimCheckpoint(insim::InsimCheckpoint),
    /// Insim circle
    InsimCircle(insim::InsimCircle),
    /// Restrited area / circle
    RestrictedArea(marshal::RestrictedArea),
    /// Route checker
    RouteChecker(marshal::RouteChecker),

    /// ChalkLine
    ChalkLine(chalk::Chalk),
    /// ChalkLine2
    ChalkLine2(chalk::Chalk),
    /// ChalkAhead
    ChalkAhead(chalk::Chalk),
    /// ChalkAhead2
    ChalkAhead2(chalk::Chalk),
    /// ChalkLeft
    ChalkLeft(chalk::Chalk),
    /// ChalkLeft2
    ChalkLeft2(chalk::Chalk),
    /// ChalkLeft3
    ChalkLeft3(chalk::Chalk),
    /// ChalkRight
    ChalkRight(chalk::Chalk),
    /// ChalkRight2
    ChalkRight2(chalk::Chalk),
    /// ChalkRight3
    ChalkRight3(chalk::Chalk),
    /// Painted Letters
    PaintLetters(painted::Letters),
    /// Painted Arrows
    PaintArrows(painted::Arrows),
    /// Cone1
    Cone1(cones::Cone),
    /// Cone2
    Cone2(cones::Cone),
    /// ConeTall1
    ConeTall1(cones::Cone),
    /// ConeTall2
    ConeTall2(cones::Cone),
    /// Cone Pointer
    ConePointer(cones::Cone),
    /// Tyre Single
    TyreSingle(tyres::Tyres),
    /// Tyre Stack2
    TyreStack2(tyres::Tyres),
    /// Tyre Stack3
    TyreStack3(tyres::Tyres),
    /// Tyre Stack4
    TyreStack4(tyres::Tyres),
    /// Tyre Single Big
    TyreSingleBig(tyres::Tyres),
    /// Tyre Stack2 Big
    TyreStack2Big(tyres::Tyres),
    /// Tyre Stack3 Big
    TyreStack3Big(tyres::Tyres),
    /// Tyre Stack4 Big
    TyreStack4Big(tyres::Tyres),
    /// Corner Marker
    MarkerCorner(marker::MarkerCorner),
    /// Distance Marker
    MarkerDistance(marker::MarkerDistance),
    /// Letterboard WY
    LetterboardWY(letterboard_wy::LetterboardWY),
    /// Letterboard RB
    LetterboardRB(letterboard_rb::LetterboardRB),
    /// Armco1
    Armco1(armco::Armco),
    /// Armco3
    Armco3(armco::Armco),
    /// Armco5
    Armco5(armco::Armco),
    /// Barrier Long
    BarrierLong(barrier::Barrier),
    /// Barrier Red
    BarrierRed(barrier::Barrier),
    /// Barrier White
    BarrierWhite(barrier::Barrier),
    /// Banner
    Banner(banner::Banner),
    /// Ramp1
    Ramp1(ramp::Ramp),
    /// Ramp2
    Ramp2(ramp::Ramp),
    /// Vehicle SUV
    VehicleSUV(vehicle_suv::VehicleSUV),
    /// Vehicle Van
    VehicleVan(vehicle_van::VehicleVan),
    /// Vehicle Truck
    VehicleTruck(vehicle_truck::VehicleTruck),
    /// Vehicle Ambulance
    VehicleAmbulance(vehicle_ambulance::VehicleAmbulance),
    /// Kerb
    Kerb(kerb::Kerb),
    /// Post
    Post(post::Post),
    /// Marquee
    Marquee(marquee::Marquee),
    /// Bale
    Bale(bale::Bale),
    /// Speed hump 10m
    SpeedHump10M(speed_hump::SpeedHump),
    /// Speed hump 6m
    SpeedHump6M(speed_hump::SpeedHump),
    /// Speed hump 2m
    SpeedHump2M(speed_hump::SpeedHump),
    /// Speed hump 1m
    SpeedHump1M(speed_hump::SpeedHump),
    /// Bin1
    Bin1(bin1::Bin1),
    /// Bin2
    Bin2(bin2::Bin2),
    /// Railing1
    Railing1(railing::Railing),
    /// Railing2
    Railing2(railing::Railing),
    /// Start lights 1
    StartLights1(start_lights::StartLights),
    /// Start lights 2
    StartLights2(start_lights::StartLights),
    /// Start lights 3
    StartLights3(start_lights::StartLights),
    /// Metal Sign
    SignMetal(sign_metal::SignMetal),
    /// ChevronLeft
    ChevronLeft(chevron::Chevron),
    /// ChevronRight
    ChevronRight(chevron::Chevron),
    /// Speed Sign
    SignSpeed(sign_speed::SignSpeed),
    /// Concrete Slab
    ConcreteSlab(concrete::ConcreteSlab),
    /// Concrete Ramp
    ConcreteRamp(concrete::ConcreteRamp),
    /// Concrete Wall
    ConcreteWall(concrete::ConcreteWall),
    /// Concrete Pillar
    ConcretePillar(concrete::ConcretePillar),
    /// Concrete Slab Wall
    ConcreteSlabWall(concrete::ConcreteSlabWall),
    /// Concrete Ramp Wall
    ConcreteRampWall(concrete::ConcreteRampWall),
    /// Concrete Short Slab Wall
    ConcreteShortSlabWall(concrete::ConcreteShortSlabWall),
    /// Concrete Wedge
    ConcreteWedge(concrete::ConcreteWedge),
    /// Start position
    StartPosition(start_position::StartPosition),
    /// Pit Startpoint
    PitStartPoint(pit_start_point::PitStartPoint),
    /// Pit stop box
    PitStopBox(pit::PitStopBox),
}

impl Default for ObjectKind {
    fn default() -> Self {
        todo!()
    }
}

impl ObjectKind {
    /// Encode to wire with index
    fn to_wire(&self) -> Result<(u8, ObjectWire), EncodeError> {
        match self {
            ObjectKind::Control(control) => {
                let wire = control.encode()?;
                Ok((0, wire))
            },
            ObjectKind::Marshal(marshal) => {
                let wire = marshal.encode()?;
                Ok((240, wire))
            },
            ObjectKind::InsimCheckpoint(insim_checkpoint) => {
                let wire = insim_checkpoint.encode()?;
                Ok((252, wire))
            },
            ObjectKind::InsimCircle(insim_circle) => {
                let wire = insim_circle.encode()?;
                Ok((253, wire))
            },
            ObjectKind::RestrictedArea(restricted_area) => {
                let wire = restricted_area.encode()?;
                Ok((254, wire))
            },
            ObjectKind::RouteChecker(route_checker) => {
                let wire = route_checker.encode()?;
                Ok((255, wire))
            },
            ObjectKind::ChalkLine(chalk) => {
                let wire = chalk.to_wire()?;
                Ok((4, wire))
            },
            ObjectKind::ChalkLine2(chalk) => {
                let wire = chalk.to_wire()?;
                Ok((5, wire))
            },
            ObjectKind::ChalkAhead(chalk) => {
                let wire = chalk.to_wire()?;
                Ok((6, wire))
            },
            ObjectKind::ChalkAhead2(chalk) => {
                let wire = chalk.to_wire()?;
                Ok((7, wire))
            },
            ObjectKind::ChalkLeft(chalk) => {
                let wire = chalk.to_wire()?;
                Ok((8, wire))
            },
            ObjectKind::ChalkLeft2(chalk) => {
                let wire = chalk.to_wire()?;
                Ok((9, wire))
            },
            ObjectKind::ChalkLeft3(chalk) => {
                let wire = chalk.to_wire()?;
                Ok((10, wire))
            },
            ObjectKind::ChalkRight(chalk) => {
                let wire = chalk.to_wire()?;
                Ok((11, wire))
            },
            ObjectKind::ChalkRight2(chalk) => {
                let wire = chalk.to_wire()?;
                Ok((12, wire))
            },
            ObjectKind::ChalkRight3(chalk) => {
                let wire = chalk.to_wire()?;
                Ok((13, wire))
            },
            ObjectKind::PaintLetters(letters) => {
                let wire = letters.to_wire()?;
                Ok((16, wire))
            },
            ObjectKind::PaintArrows(arrows) => {
                let wire = arrows.to_wire()?;
                Ok((17, wire))
            },
            ObjectKind::Cone1(cone1) => {
                let wire = cone1.to_wire()?;
                Ok((20, wire))
            },
            ObjectKind::Cone2(cone2) => {
                let wire = cone2.to_wire()?;
                Ok((21, wire))
            },
            ObjectKind::ConeTall1(cone_tall1) => {
                let wire = cone_tall1.to_wire()?;
                Ok((32, wire))
            },
            ObjectKind::ConeTall2(cone_tall2) => {
                let wire = cone_tall2.to_wire()?;
                Ok((33, wire))
            },
            ObjectKind::ConePointer(cone_pointer) => {
                let wire = cone_pointer.to_wire()?;
                Ok((40, wire))
            },
            ObjectKind::TyreSingle(tyre) => {
                let wire = tyre.to_wire()?;
                Ok((48, wire))
            },
            ObjectKind::TyreStack2(tyre) => {
                let wire = tyre.to_wire()?;
                Ok((49, wire))
            },
            ObjectKind::TyreStack3(tyre) => {
                let wire = tyre.to_wire()?;
                Ok((50, wire))
            },
            ObjectKind::TyreStack4(tyre) => {
                let wire = tyre.to_wire()?;
                Ok((51, wire))
            },
            ObjectKind::TyreSingleBig(tyre) => {
                let wire = tyre.to_wire()?;
                Ok((52, wire))
            },
            ObjectKind::TyreStack2Big(tyre) => {
                let wire = tyre.to_wire()?;
                Ok((53, wire))
            },
            ObjectKind::TyreStack3Big(tyre) => {
                let wire = tyre.to_wire()?;
                Ok((54, wire))
            },
            ObjectKind::TyreStack4Big(tyre) => {
                let wire = tyre.to_wire()?;
                Ok((55, wire))
            },
            ObjectKind::MarkerCorner(marker_corner) => {
                let wire = marker_corner.to_wire()?;
                Ok((62, wire))
            },
            ObjectKind::MarkerDistance(marker_distance) => {
                let wire = marker_distance.to_wire()?;
                Ok((84, wire))
            },
            ObjectKind::LetterboardWY(letterboard_wy) => {
                let wire = letterboard_wy.to_wire()?;
                Ok((92, wire))
            },
            ObjectKind::LetterboardRB(letterboard_rb) => {
                let wire = letterboard_rb.to_wire()?;
                Ok((93, wire))
            },
            ObjectKind::Armco1(armco1) => {
                let wire = armco1.to_wire()?;
                Ok((96, wire))
            },
            ObjectKind::Armco3(armco3) => {
                let wire = armco3.to_wire()?;
                Ok((97, wire))
            },
            ObjectKind::Armco5(armco5) => {
                let wire = armco5.to_wire()?;
                Ok((98, wire))
            },
            ObjectKind::BarrierLong(barrier) => {
                let wire = barrier.to_wire()?;
                Ok((104, wire))
            },
            ObjectKind::BarrierRed(barrier) => {
                let wire = barrier.to_wire()?;
                Ok((105, wire))
            },
            ObjectKind::BarrierWhite(barrier) => {
                let wire = barrier.to_wire()?;
                Ok((106, wire))
            },
            ObjectKind::Banner(banner) => {
                let wire = banner.to_wire()?;
                Ok((112, wire))
            },
            ObjectKind::Ramp1(ramp1) => {
                let wire = ramp1.to_wire()?;
                Ok((120, wire))
            },
            ObjectKind::Ramp2(ramp2) => {
                let wire = ramp2.to_wire()?;
                Ok((121, wire))
            },
            ObjectKind::VehicleSUV(veh) => {
                let wire = veh.to_wire()?;
                Ok((124, wire))
            },
            ObjectKind::VehicleVan(veh) => {
                let wire = veh.to_wire()?;
                Ok((125, wire))
            },
            ObjectKind::VehicleTruck(veh) => {
                let wire = veh.to_wire()?;
                Ok((126, wire))
            },
            ObjectKind::VehicleAmbulance(veh) => {
                let wire = veh.to_wire()?;
                Ok((127, wire))
            },
            ObjectKind::SpeedHump10M(speed_hump) => {
                let wire = speed_hump.to_wire()?;
                Ok((128, wire))
            },
            ObjectKind::SpeedHump6M(speed_hump) => {
                let wire = speed_hump.to_wire()?;
                Ok((129, wire))
            },
            ObjectKind::SpeedHump2M(speed_hump) => {
                let wire = speed_hump.to_wire()?;
                Ok((130, wire))
            },
            ObjectKind::SpeedHump1M(speed_hump) => {
                let wire = speed_hump.to_wire()?;
                Ok((131, wire))
            },
            ObjectKind::Kerb(kerb) => {
                let wire = kerb.to_wire()?;
                Ok((132, wire))
            },
            ObjectKind::Post(post) => {
                let wire = post.to_wire()?;
                Ok((136, wire))
            },
            ObjectKind::Marquee(marquee) => {
                let wire = marquee.to_wire()?;
                Ok((140, wire))
            },
            ObjectKind::Bale(bale) => {
                let wire = bale.to_wire()?;
                Ok((144, wire))
            },
            ObjectKind::Bin1(bin1) => {
                let wire = bin1.to_wire()?;
                Ok((145, wire))
            },
            ObjectKind::Bin2(bin2) => {
                let wire = bin2.to_wire()?;
                Ok((146, wire))
            },
            ObjectKind::Railing1(railing1) => {
                let wire = railing1.to_wire()?;
                Ok((147, wire))
            },
            ObjectKind::Railing2(railing2) => {
                let wire = railing2.to_wire()?;
                Ok((148, wire))
            },
            ObjectKind::StartLights1(start_lights1) => {
                let wire = start_lights1.to_wire()?;
                Ok((149, wire))
            },
            ObjectKind::StartLights2(start_lights2) => {
                let wire = start_lights2.to_wire()?;
                Ok((150, wire))
            },
            ObjectKind::StartLights3(start_lights3) => {
                let wire = start_lights3.to_wire()?;
                Ok((151, wire))
            },
            ObjectKind::SignMetal(sign_metal) => {
                let wire = sign_metal.to_wire()?;
                Ok((160, wire))
            },
            ObjectKind::ChevronLeft(chevron) => {
                let wire = chevron.to_wire()?;
                Ok((164, wire))
            },
            ObjectKind::ChevronRight(chevron) => {
                let wire = chevron.to_wire()?;
                Ok((165, wire))
            },
            ObjectKind::SignSpeed(sign_speed) => {
                let wire = sign_speed.to_wire()?;
                Ok((168, wire))
            },
            ObjectKind::ConcreteSlab(concrete_slab) => {
                let wire = concrete_slab.to_wire()?;
                Ok((172, wire))
            },
            ObjectKind::ConcreteRamp(concrete_ramp) => {
                let wire = concrete_ramp.to_wire()?;
                Ok((173, wire))
            },
            ObjectKind::ConcreteWall(concrete_wall) => {
                let wire = concrete_wall.to_wire()?;
                Ok((174, wire))
            },
            ObjectKind::ConcretePillar(concrete_pillar) => {
                let wire = concrete_pillar.to_wire()?;
                Ok((175, wire))
            },
            ObjectKind::ConcreteSlabWall(concrete_slab_wall) => {
                let wire = concrete_slab_wall.to_wire()?;
                Ok((176, wire))
            },
            ObjectKind::ConcreteRampWall(concrete_ramp_wall) => {
                let wire = concrete_ramp_wall.to_wire()?;
                Ok((177, wire))
            },
            ObjectKind::ConcreteShortSlabWall(concrete_short_slab_wall) => {
                let wire = concrete_short_slab_wall.to_wire()?;
                Ok((178, wire))
            },
            ObjectKind::ConcreteWedge(concrete_wedge) => {
                let wire = concrete_wedge.to_wire()?;
                Ok((179, wire))
            },
            ObjectKind::StartPosition(start_position) => {
                let wire = start_position.to_wire()?;
                Ok((184, wire))
            },
            ObjectKind::PitStartPoint(pit_start_point) => {
                let wire = pit_start_point.to_wire()?;
                Ok((185, wire))
            },
            ObjectKind::PitStopBox(pit_stop_box) => {
                let wire = pit_stop_box.to_wire()?;
                Ok((186, wire))
            },
        }
    }

    /// Decode from wire with index
    fn from_wire(index: u8, wire: ObjectWire) -> Result<Self, DecodeError> {
        match index {
            0 => Ok(ObjectKind::Control(control::Control::decode(wire)?)),
            240 => Ok(ObjectKind::Marshal(marshal::Marshal::decode(wire)?)),
            252 => Ok(ObjectKind::InsimCheckpoint(insim::InsimCheckpoint::decode(
                wire,
            )?)),
            253 => Ok(ObjectKind::InsimCircle(insim::InsimCircle::decode(wire)?)),
            254 => Ok(ObjectKind::RestrictedArea(marshal::RestrictedArea::decode(
                wire,
            )?)),
            255 => Ok(ObjectKind::RouteChecker(marshal::RouteChecker::decode(
                wire,
            )?)),

            4 => Ok(ObjectKind::ChalkLine(chalk::Chalk::from_wire(wire)?)),
            5 => Ok(ObjectKind::ChalkLine2(chalk::Chalk::from_wire(wire)?)),
            6 => Ok(ObjectKind::ChalkAhead(chalk::Chalk::from_wire(wire)?)),
            7 => Ok(ObjectKind::ChalkAhead2(chalk::Chalk::from_wire(wire)?)),
            8 => Ok(ObjectKind::ChalkLeft(chalk::Chalk::from_wire(wire)?)),
            9 => Ok(ObjectKind::ChalkLeft2(chalk::Chalk::from_wire(wire)?)),
            10 => Ok(ObjectKind::ChalkLeft3(chalk::Chalk::from_wire(wire)?)),
            11 => Ok(ObjectKind::ChalkRight(chalk::Chalk::from_wire(wire)?)),
            12 => Ok(ObjectKind::ChalkRight2(chalk::Chalk::from_wire(wire)?)),
            13 => Ok(ObjectKind::ChalkRight3(chalk::Chalk::from_wire(wire)?)),
            16 => Ok(ObjectKind::PaintLetters(painted::Letters::from_wire(wire)?)),
            17 => Ok(ObjectKind::PaintArrows(painted::Arrows::from_wire(wire)?)),
            20 => Ok(ObjectKind::Cone1(cones::Cone::from_wire(wire)?)),
            21 => Ok(ObjectKind::Cone2(cones::Cone::from_wire(wire)?)),
            32 => Ok(ObjectKind::ConeTall1(cones::Cone::from_wire(wire)?)),
            33 => Ok(ObjectKind::ConeTall2(cones::Cone::from_wire(wire)?)),
            40 => Ok(ObjectKind::ConePointer(cones::Cone::from_wire(wire)?)),

            48 => Ok(ObjectKind::TyreSingle(tyres::Tyres::from_wire(wire)?)),
            49 => Ok(ObjectKind::TyreStack2(tyres::Tyres::from_wire(wire)?)),
            50 => Ok(ObjectKind::TyreStack3(tyres::Tyres::from_wire(wire)?)),
            51 => Ok(ObjectKind::TyreStack4(tyres::Tyres::from_wire(wire)?)),
            52 => Ok(ObjectKind::TyreSingleBig(tyres::Tyres::from_wire(wire)?)),
            53 => Ok(ObjectKind::TyreStack2Big(tyres::Tyres::from_wire(wire)?)),
            54 => Ok(ObjectKind::TyreStack3Big(tyres::Tyres::from_wire(wire)?)),
            55 => Ok(ObjectKind::TyreStack4Big(tyres::Tyres::from_wire(wire)?)),

            62 => Ok(ObjectKind::MarkerCorner(marker::MarkerCorner::from_wire(
                wire,
            )?)),
            84 => Ok(ObjectKind::MarkerDistance(
                marker::MarkerDistance::from_wire(wire)?,
            )),
            92 => Ok(ObjectKind::LetterboardWY(
                letterboard_wy::LetterboardWY::from_wire(wire)?,
            )),
            93 => Ok(ObjectKind::LetterboardRB(
                letterboard_rb::LetterboardRB::from_wire(wire)?,
            )),
            96 => Ok(ObjectKind::Armco1(armco::Armco::from_wire(wire)?)),
            97 => Ok(ObjectKind::Armco3(armco::Armco::from_wire(wire)?)),
            98 => Ok(ObjectKind::Armco5(armco::Armco::from_wire(wire)?)),
            104 => Ok(ObjectKind::BarrierLong(barrier::Barrier::from_wire(wire)?)),
            105 => Ok(ObjectKind::BarrierRed(barrier::Barrier::from_wire(wire)?)),
            106 => Ok(ObjectKind::BarrierWhite(barrier::Barrier::from_wire(wire)?)),
            112 => Ok(ObjectKind::Banner(banner::Banner::from_wire(wire)?)),
            120 => Ok(ObjectKind::Ramp1(ramp::Ramp::from_wire(wire)?)),
            121 => Ok(ObjectKind::Ramp2(ramp::Ramp::from_wire(wire)?)),
            124 => Ok(ObjectKind::VehicleSUV(vehicle_suv::VehicleSUV::from_wire(
                wire,
            )?)),
            125 => Ok(ObjectKind::VehicleVan(vehicle_van::VehicleVan::from_wire(
                wire,
            )?)),
            126 => Ok(ObjectKind::VehicleTruck(
                vehicle_truck::VehicleTruck::from_wire(wire)?,
            )),
            127 => Ok(ObjectKind::VehicleAmbulance(
                vehicle_ambulance::VehicleAmbulance::from_wire(wire)?,
            )),
            128 => Ok(ObjectKind::SpeedHump10M(speed_hump::SpeedHump::from_wire(
                wire,
            )?)),
            129 => Ok(ObjectKind::SpeedHump6M(speed_hump::SpeedHump::from_wire(
                wire,
            )?)),
            130 => Ok(ObjectKind::SpeedHump2M(speed_hump::SpeedHump::from_wire(
                wire,
            )?)),
            131 => Ok(ObjectKind::SpeedHump1M(speed_hump::SpeedHump::from_wire(
                wire,
            )?)),
            132 => Ok(ObjectKind::Kerb(kerb::Kerb::from_wire(wire)?)),
            136 => Ok(ObjectKind::Post(post::Post::from_wire(wire)?)),
            140 => Ok(ObjectKind::Marquee(marquee::Marquee::from_wire(wire)?)),
            144 => Ok(ObjectKind::Bale(bale::Bale::from_wire(wire)?)),
            145 => Ok(ObjectKind::Bin1(bin1::Bin1::from_wire(wire)?)),
            146 => Ok(ObjectKind::Bin2(bin2::Bin2::from_wire(wire)?)),
            147 => Ok(ObjectKind::Railing1(railing::Railing::from_wire(wire)?)),
            148 => Ok(ObjectKind::Railing2(railing::Railing::from_wire(wire)?)),
            149 => Ok(ObjectKind::StartLights1(
                start_lights::StartLights::from_wire(wire)?,
            )),
            150 => Ok(ObjectKind::StartLights2(
                start_lights::StartLights::from_wire(wire)?,
            )),
            151 => Ok(ObjectKind::StartLights3(
                start_lights::StartLights::from_wire(wire)?,
            )),
            160 => Ok(ObjectKind::SignMetal(sign_metal::SignMetal::from_wire(
                wire,
            )?)),
            164 => Ok(ObjectKind::ChevronLeft(chevron::Chevron::from_wire(wire)?)),
            165 => Ok(ObjectKind::ChevronRight(chevron::Chevron::from_wire(wire)?)),
            168 => Ok(ObjectKind::SignSpeed(sign_speed::SignSpeed::from_wire(
                wire,
            )?)),
            172 => Ok(ObjectKind::ConcreteSlab(concrete::ConcreteSlab::from_wire(
                wire,
            )?)),
            173 => Ok(ObjectKind::ConcreteRamp(concrete::ConcreteRamp::from_wire(
                wire,
            )?)),
            174 => Ok(ObjectKind::ConcreteWall(concrete::ConcreteWall::from_wire(
                wire,
            )?)),
            175 => Ok(ObjectKind::ConcretePillar(
                concrete::ConcretePillar::from_wire(wire)?,
            )),
            176 => Ok(ObjectKind::ConcreteSlabWall(
                concrete::ConcreteSlabWall::from_wire(wire)?,
            )),
            177 => Ok(ObjectKind::ConcreteRampWall(
                concrete::ConcreteRampWall::from_wire(wire)?,
            )),
            178 => Ok(ObjectKind::ConcreteShortSlabWall(
                concrete::ConcreteShortSlabWall::from_wire(wire)?,
            )),
            179 => Ok(ObjectKind::ConcreteWedge(
                concrete::ConcreteWedge::from_wire(wire)?,
            )),
            184 => Ok(ObjectKind::StartPosition(
                start_position::StartPosition::from_wire(wire)?,
            )),
            185 => Ok(ObjectKind::PitStartPoint(
                pit_start_point::PitStartPoint::from_wire(wire)?,
            )),
            186 => Ok(ObjectKind::PitStopBox(pit::PitStopBox::from_wire(wire)?)),
            _ => Err(DecodeError::NoVariantMatch {
                found: index as u64,
            }),
        }
    }
}

impl Decode for ObjectInfo {
    fn decode(buf: &mut bytes::Bytes) -> Result<Self, DecodeError> {
        let x = i16::decode(buf)?;
        let y = i16::decode(buf)?;
        let z = u8::decode(buf)?;
        let xyz = ObjectCoordinate { x, y, z };

        let flags = u8::decode(buf)?;
        let index = u8::decode(buf)?;
        let heading = u8::decode(buf)?;

        let wire = ObjectWire { flags, heading };
        let kind = ObjectKind::from_wire(index, wire)?;

        Ok(Self { xyz, kind })
    }
}

impl ObjectKind {
    /// Get heading if this object has one
    pub fn heading(&self) -> Option<crate::heading::Heading> {
        match self {
            ObjectKind::Control(c) => Some(c.heading),
            ObjectKind::Marshal(m) => Some(m.heading),
            ObjectKind::InsimCheckpoint(ic) => Some(ic.heading),
            ObjectKind::ChalkLine(c) => Some(c.heading),
            ObjectKind::ChalkLine2(c) => Some(c.heading),
            ObjectKind::ChalkAhead(c) => Some(c.heading),
            ObjectKind::ChalkAhead2(c) => Some(c.heading),
            ObjectKind::ChalkLeft(c) => Some(c.heading),
            ObjectKind::ChalkLeft2(c) => Some(c.heading),
            ObjectKind::ChalkLeft3(c) => Some(c.heading),
            ObjectKind::ChalkRight(c) => Some(c.heading),
            ObjectKind::ChalkRight2(c) => Some(c.heading),
            ObjectKind::ChalkRight3(c) => Some(c.heading),
            ObjectKind::PaintLetters(l) => Some(l.heading),
            ObjectKind::PaintArrows(a) => Some(a.heading),
            ObjectKind::Cone1(c) => Some(c.heading),
            ObjectKind::Cone2(c) => Some(c.heading),
            ObjectKind::ConeTall1(c) => Some(c.heading),
            ObjectKind::ConeTall2(c) => Some(c.heading),
            ObjectKind::ConePointer(cp) => Some(cp.heading),
            ObjectKind::TyreSingle(t) => Some(t.heading),
            ObjectKind::TyreStack2(t) => Some(t.heading),
            ObjectKind::TyreStack3(t) => Some(t.heading),
            ObjectKind::TyreStack4(t) => Some(t.heading),
            ObjectKind::TyreSingleBig(t) => Some(t.heading),
            ObjectKind::TyreStack2Big(t) => Some(t.heading),
            ObjectKind::TyreStack3Big(t) => Some(t.heading),
            ObjectKind::TyreStack4Big(t) => Some(t.heading),
            ObjectKind::MarkerCorner(m) => Some(m.heading),
            ObjectKind::MarkerDistance(m) => Some(m.heading),
            ObjectKind::LetterboardWY(l) => Some(l.heading),
            ObjectKind::LetterboardRB(l) => Some(l.heading),
            ObjectKind::Armco1(a) => Some(a.heading),
            ObjectKind::Armco3(a) => Some(a.heading),
            ObjectKind::Armco5(a) => Some(a.heading),
            ObjectKind::BarrierLong(b) => Some(b.heading),
            ObjectKind::BarrierRed(b) => Some(b.heading),
            ObjectKind::BarrierWhite(b) => Some(b.heading),
            ObjectKind::Banner(b) => Some(b.heading),
            ObjectKind::Ramp1(r) => Some(r.heading),
            ObjectKind::Ramp2(r) => Some(r.heading),
            ObjectKind::VehicleSUV(v) => Some(v.heading),
            ObjectKind::VehicleVan(v) => Some(v.heading),
            ObjectKind::VehicleTruck(v) => Some(v.heading),
            ObjectKind::VehicleAmbulance(v) => Some(v.heading),
            ObjectKind::SpeedHump10M(s) => Some(s.heading),
            ObjectKind::SpeedHump6M(s) => Some(s.heading),
            ObjectKind::SpeedHump2M(s) => Some(s.heading),
            ObjectKind::SpeedHump1M(s) => Some(s.heading),
            ObjectKind::Kerb(k) => Some(k.heading),
            ObjectKind::Post(p) => Some(p.heading),
            ObjectKind::Marquee(m) => Some(m.heading),
            ObjectKind::Bale(b) => Some(b.heading),
            ObjectKind::Bin1(b) => Some(b.heading),
            ObjectKind::Bin2(b) => Some(b.heading),
            ObjectKind::Railing1(r) => Some(r.heading),
            ObjectKind::Railing2(r) => Some(r.heading),
            ObjectKind::StartLights1(s) => Some(s.heading),
            ObjectKind::StartLights2(s) => Some(s.heading),
            ObjectKind::StartLights3(s) => Some(s.heading),
            ObjectKind::SignMetal(s) => Some(s.heading),
            ObjectKind::SignSpeed(s) => Some(s.heading),
            ObjectKind::ConcreteSlab(c) => Some(c.heading),
            ObjectKind::ConcreteRamp(c) => Some(c.heading),
            ObjectKind::ConcreteWall(c) => Some(c.heading),
            ObjectKind::ConcretePillar(c) => Some(c.heading),
            ObjectKind::ConcreteSlabWall(c) => Some(c.heading),
            ObjectKind::ConcreteRampWall(c) => Some(c.heading),
            ObjectKind::ConcreteShortSlabWall(c) => Some(c.heading),
            ObjectKind::ConcreteWedge(c) => Some(c.heading),
            ObjectKind::StartPosition(s) => Some(s.heading),
            ObjectKind::PitStartPoint(p) => Some(p.heading),
            ObjectKind::PitStopBox(p) => Some(p.heading),
            ObjectKind::InsimCircle(_) => None,
            ObjectKind::RestrictedArea(_) => None,
            ObjectKind::RouteChecker(_) => None,
            ObjectKind::ChevronLeft(c) => Some(c.heading),
            ObjectKind::ChevronRight(c) => Some(c.heading),
        }
    }

    /// Get floating flag if this object has one
    pub fn is_floating(&self) -> Option<bool> {
        match self {
            ObjectKind::Control(c) => Some(c.floating),
            ObjectKind::Marshal(m) => Some(m.floating),
            ObjectKind::InsimCheckpoint(ic) => Some(ic.floating),
            ObjectKind::InsimCircle(ic) => Some(ic.floating),
            ObjectKind::RestrictedArea(ra) => Some(ra.floating),
            ObjectKind::RouteChecker(rc) => Some(rc.floating),
            ObjectKind::ChalkLine(c) => Some(c.floating),
            ObjectKind::ChalkLine2(c) => Some(c.floating),
            ObjectKind::ChalkAhead(c) => Some(c.floating),
            ObjectKind::ChalkAhead2(c) => Some(c.floating),
            ObjectKind::ChalkLeft(c) => Some(c.floating),
            ObjectKind::ChalkLeft2(c) => Some(c.floating),
            ObjectKind::ChalkLeft3(c) => Some(c.floating),
            ObjectKind::ChalkRight(c) => Some(c.floating),
            ObjectKind::ChalkRight2(c) => Some(c.floating),
            ObjectKind::ChalkRight3(c) => Some(c.floating),
            ObjectKind::PaintLetters(l) => Some(l.floating),
            ObjectKind::PaintArrows(a) => Some(a.floating),
            ObjectKind::Cone1(c) => Some(c.floating),
            ObjectKind::Cone2(c) => Some(c.floating),
            ObjectKind::ConeTall1(c) => Some(c.floating),
            ObjectKind::ConeTall2(c) => Some(c.floating),
            ObjectKind::ConePointer(cp) => Some(cp.floating),
            ObjectKind::TyreSingle(t) => Some(t.floating),
            ObjectKind::TyreStack2(t) => Some(t.floating),
            ObjectKind::TyreStack3(t) => Some(t.floating),
            ObjectKind::TyreStack4(t) => Some(t.floating),
            ObjectKind::TyreSingleBig(t) => Some(t.floating),
            ObjectKind::TyreStack2Big(t) => Some(t.floating),
            ObjectKind::TyreStack3Big(t) => Some(t.floating),
            ObjectKind::TyreStack4Big(t) => Some(t.floating),
            ObjectKind::MarkerCorner(m) => Some(m.floating),
            ObjectKind::MarkerDistance(m) => Some(m.floating),
            ObjectKind::LetterboardWY(l) => Some(l.floating),
            ObjectKind::LetterboardRB(l) => Some(l.floating),
            ObjectKind::Armco1(a) => Some(a.floating),
            ObjectKind::Armco3(a) => Some(a.floating),
            ObjectKind::Armco5(a) => Some(a.floating),
            ObjectKind::BarrierLong(b) => Some(b.floating),
            ObjectKind::BarrierRed(b) => Some(b.floating),
            ObjectKind::BarrierWhite(b) => Some(b.floating),
            ObjectKind::Banner(b) => Some(b.floating),
            ObjectKind::Ramp1(r) => Some(r.floating),
            ObjectKind::Ramp2(r) => Some(r.floating),
            ObjectKind::VehicleSUV(v) => Some(v.floating),
            ObjectKind::VehicleVan(v) => Some(v.floating),
            ObjectKind::VehicleTruck(v) => Some(v.floating),
            ObjectKind::VehicleAmbulance(v) => Some(v.floating),
            ObjectKind::SpeedHump10M(s) => Some(s.floating),
            ObjectKind::SpeedHump6M(s) => Some(s.floating),
            ObjectKind::SpeedHump2M(s) => Some(s.floating),
            ObjectKind::SpeedHump1M(s) => Some(s.floating),
            ObjectKind::Kerb(k) => Some(k.floating),
            ObjectKind::Post(p) => Some(p.floating),
            ObjectKind::Marquee(m) => Some(m.floating),
            ObjectKind::Bale(b) => Some(b.floating),
            ObjectKind::Bin1(b) => Some(b.floating),
            ObjectKind::Bin2(b) => Some(b.floating),
            ObjectKind::Railing1(r) => Some(r.floating),
            ObjectKind::Railing2(r) => Some(r.floating),
            ObjectKind::StartLights1(s) => Some(s.floating),
            ObjectKind::StartLights2(s) => Some(s.floating),
            ObjectKind::StartLights3(s) => Some(s.floating),
            ObjectKind::SignMetal(s) => Some(s.floating),
            ObjectKind::SignSpeed(s) => Some(s.floating),
            ObjectKind::ConcreteSlab(_) => None, // Concrete objects always float (per spec)
            ObjectKind::ConcreteRamp(_) => None,
            ObjectKind::ConcreteWall(_) => None,
            ObjectKind::ConcretePillar(_) => None,
            ObjectKind::ConcreteSlabWall(_) => None,
            ObjectKind::ConcreteRampWall(_) => None,
            ObjectKind::ConcreteShortSlabWall(_) => None,
            ObjectKind::ConcreteWedge(_) => None,
            ObjectKind::StartPosition(s) => Some(s.floating),
            ObjectKind::PitStartPoint(p) => Some(p.floating),
            ObjectKind::PitStopBox(p) => Some(p.floating),
            ObjectKind::ChevronLeft(p) => Some(p.floating),
            ObjectKind::ChevronRight(p) => Some(p.floating),
        }
    }
}

impl Encode for ObjectInfo {
    fn encode(&self, buf: &mut bytes::BytesMut) -> Result<(), EncodeError> {
        self.xyz.x.encode(buf)?;
        self.xyz.y.encode(buf)?;
        self.xyz.z.encode(buf)?;
        let (index, wire) = self.kind.to_wire()?;
        wire.flags.encode(buf)?;
        index.encode(buf)?;
        wire.heading.encode(buf)?;

        Ok(())
    }
}
