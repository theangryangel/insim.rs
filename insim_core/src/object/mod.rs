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
pub(crate) struct ObjectIntermediate {
    /// XYZ ObjectCoordinate
    pub xyz: ObjectCoordinate,
    /// Flags byte (semantics depend on object type)
    pub flags: u8,
    /// Heading/data byte (semantics depend on object type)
    pub heading: u8,
}

impl ObjectIntermediate {
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
    fn to_wire(&self) -> Result<ObjectIntermediate, EncodeError>;
    /// Decode Object from wire format
    fn from_wire(wire: ObjectIntermediate) -> Result<Self, DecodeError>;
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Layout Object Position
pub struct ObjectCoordinate {
    pub x: i16,
    pub y: i16,
    pub z: u8,
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

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[non_exhaustive]
/// Layout Object
pub enum ObjectInfo {
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

impl Decode for ObjectInfo {
    fn decode(buf: &mut bytes::Bytes) -> Result<Self, DecodeError> {
        let x = i16::decode(buf)?;
        let y = i16::decode(buf)?;
        let z = u8::decode(buf)?;
        let xyz = ObjectCoordinate { x, y, z };

        let flags = u8::decode(buf)?;
        let index = u8::decode(buf)?;
        let heading = u8::decode(buf)?;

        let wire = ObjectIntermediate { xyz, flags, heading };
        match index {
            0 => Ok(ObjectInfo::Control(control::Control::decode(wire)?)),
            240 => Ok(ObjectInfo::Marshal(marshal::Marshal::decode(wire)?)),
            252 => Ok(ObjectInfo::InsimCheckpoint(insim::InsimCheckpoint::decode(
                wire,
            )?)),
            253 => Ok(ObjectInfo::InsimCircle(insim::InsimCircle::decode(wire)?)),
            254 => Ok(ObjectInfo::RestrictedArea(marshal::RestrictedArea::decode(
                wire,
            )?)),
            255 => Ok(ObjectInfo::RouteChecker(marshal::RouteChecker::decode(
                wire,
            )?)),

            4 => Ok(ObjectInfo::ChalkLine(chalk::Chalk::from_wire(wire)?)),
            5 => Ok(ObjectInfo::ChalkLine2(chalk::Chalk::from_wire(wire)?)),
            6 => Ok(ObjectInfo::ChalkAhead(chalk::Chalk::from_wire(wire)?)),
            7 => Ok(ObjectInfo::ChalkAhead2(chalk::Chalk::from_wire(wire)?)),
            8 => Ok(ObjectInfo::ChalkLeft(chalk::Chalk::from_wire(wire)?)),
            9 => Ok(ObjectInfo::ChalkLeft2(chalk::Chalk::from_wire(wire)?)),
            10 => Ok(ObjectInfo::ChalkLeft3(chalk::Chalk::from_wire(wire)?)),
            11 => Ok(ObjectInfo::ChalkRight(chalk::Chalk::from_wire(wire)?)),
            12 => Ok(ObjectInfo::ChalkRight2(chalk::Chalk::from_wire(wire)?)),
            13 => Ok(ObjectInfo::ChalkRight3(chalk::Chalk::from_wire(wire)?)),
            16 => Ok(ObjectInfo::PaintLetters(painted::Letters::from_wire(wire)?)),
            17 => Ok(ObjectInfo::PaintArrows(painted::Arrows::from_wire(wire)?)),
            20 => Ok(ObjectInfo::Cone1(cones::Cone::from_wire(wire)?)),
            21 => Ok(ObjectInfo::Cone2(cones::Cone::from_wire(wire)?)),
            32 => Ok(ObjectInfo::ConeTall1(cones::Cone::from_wire(wire)?)),
            33 => Ok(ObjectInfo::ConeTall2(cones::Cone::from_wire(wire)?)),
            40 => Ok(ObjectInfo::ConePointer(cones::Cone::from_wire(wire)?)),

            48 => Ok(ObjectInfo::TyreSingle(tyres::Tyres::from_wire(wire)?)),
            49 => Ok(ObjectInfo::TyreStack2(tyres::Tyres::from_wire(wire)?)),
            50 => Ok(ObjectInfo::TyreStack3(tyres::Tyres::from_wire(wire)?)),
            51 => Ok(ObjectInfo::TyreStack4(tyres::Tyres::from_wire(wire)?)),
            52 => Ok(ObjectInfo::TyreSingleBig(tyres::Tyres::from_wire(wire)?)),
            53 => Ok(ObjectInfo::TyreStack2Big(tyres::Tyres::from_wire(wire)?)),
            54 => Ok(ObjectInfo::TyreStack3Big(tyres::Tyres::from_wire(wire)?)),
            55 => Ok(ObjectInfo::TyreStack4Big(tyres::Tyres::from_wire(wire)?)),

            62 => Ok(ObjectInfo::MarkerCorner(marker::MarkerCorner::from_wire(
                wire,
            )?)),
            84 => Ok(ObjectInfo::MarkerDistance(
                marker::MarkerDistance::from_wire(wire)?,
            )),
            92 => Ok(ObjectInfo::LetterboardWY(
                letterboard_wy::LetterboardWY::from_wire(wire)?,
            )),
            93 => Ok(ObjectInfo::LetterboardRB(
                letterboard_rb::LetterboardRB::from_wire(wire)?,
            )),
            96 => Ok(ObjectInfo::Armco1(armco::Armco::from_wire(wire)?)),
            97 => Ok(ObjectInfo::Armco3(armco::Armco::from_wire(wire)?)),
            98 => Ok(ObjectInfo::Armco5(armco::Armco::from_wire(wire)?)),
            104 => Ok(ObjectInfo::BarrierLong(barrier::Barrier::from_wire(wire)?)),
            105 => Ok(ObjectInfo::BarrierRed(barrier::Barrier::from_wire(wire)?)),
            106 => Ok(ObjectInfo::BarrierWhite(barrier::Barrier::from_wire(wire)?)),
            112 => Ok(ObjectInfo::Banner(banner::Banner::from_wire(wire)?)),
            120 => Ok(ObjectInfo::Ramp1(ramp::Ramp::from_wire(wire)?)),
            121 => Ok(ObjectInfo::Ramp2(ramp::Ramp::from_wire(wire)?)),
            124 => Ok(ObjectInfo::VehicleSUV(vehicle_suv::VehicleSUV::from_wire(
                wire,
            )?)),
            125 => Ok(ObjectInfo::VehicleVan(vehicle_van::VehicleVan::from_wire(
                wire,
            )?)),
            126 => Ok(ObjectInfo::VehicleTruck(
                vehicle_truck::VehicleTruck::from_wire(wire)?,
            )),
            127 => Ok(ObjectInfo::VehicleAmbulance(
                vehicle_ambulance::VehicleAmbulance::from_wire(wire)?,
            )),
            128 => Ok(ObjectInfo::SpeedHump10M(speed_hump::SpeedHump::from_wire(
                wire,
            )?)),
            129 => Ok(ObjectInfo::SpeedHump6M(speed_hump::SpeedHump::from_wire(
                wire,
            )?)),
            130 => Ok(ObjectInfo::SpeedHump2M(speed_hump::SpeedHump::from_wire(
                wire,
            )?)),
            131 => Ok(ObjectInfo::SpeedHump1M(speed_hump::SpeedHump::from_wire(
                wire,
            )?)),
            132 => Ok(ObjectInfo::Kerb(kerb::Kerb::from_wire(wire)?)),
            136 => Ok(ObjectInfo::Post(post::Post::from_wire(wire)?)),
            140 => Ok(ObjectInfo::Marquee(marquee::Marquee::from_wire(wire)?)),
            144 => Ok(ObjectInfo::Bale(bale::Bale::from_wire(wire)?)),
            145 => Ok(ObjectInfo::Bin1(bin1::Bin1::from_wire(wire)?)),
            146 => Ok(ObjectInfo::Bin2(bin2::Bin2::from_wire(wire)?)),
            147 => Ok(ObjectInfo::Railing1(railing::Railing::from_wire(wire)?)),
            148 => Ok(ObjectInfo::Railing2(railing::Railing::from_wire(wire)?)),
            149 => Ok(ObjectInfo::StartLights1(
                start_lights::StartLights::from_wire(wire)?,
            )),
            150 => Ok(ObjectInfo::StartLights2(
                start_lights::StartLights::from_wire(wire)?,
            )),
            151 => Ok(ObjectInfo::StartLights3(
                start_lights::StartLights::from_wire(wire)?,
            )),
            160 => Ok(ObjectInfo::SignMetal(sign_metal::SignMetal::from_wire(
                wire,
            )?)),
            164 => Ok(ObjectInfo::ChevronLeft(chevron::Chevron::from_wire(wire)?)),
            165 => Ok(ObjectInfo::ChevronRight(chevron::Chevron::from_wire(wire)?)),
            168 => Ok(ObjectInfo::SignSpeed(sign_speed::SignSpeed::from_wire(
                wire,
            )?)),
            172 => Ok(ObjectInfo::ConcreteSlab(concrete::ConcreteSlab::from_wire(
                wire,
            )?)),
            173 => Ok(ObjectInfo::ConcreteRamp(concrete::ConcreteRamp::from_wire(
                wire,
            )?)),
            174 => Ok(ObjectInfo::ConcreteWall(concrete::ConcreteWall::from_wire(
                wire,
            )?)),
            175 => Ok(ObjectInfo::ConcretePillar(
                concrete::ConcretePillar::from_wire(wire)?,
            )),
            176 => Ok(ObjectInfo::ConcreteSlabWall(
                concrete::ConcreteSlabWall::from_wire(wire)?,
            )),
            177 => Ok(ObjectInfo::ConcreteRampWall(
                concrete::ConcreteRampWall::from_wire(wire)?,
            )),
            178 => Ok(ObjectInfo::ConcreteShortSlabWall(
                concrete::ConcreteShortSlabWall::from_wire(wire)?,
            )),
            179 => Ok(ObjectInfo::ConcreteWedge(
                concrete::ConcreteWedge::from_wire(wire)?,
            )),
            184 => Ok(ObjectInfo::StartPosition(
                start_position::StartPosition::from_wire(wire)?,
            )),
            185 => Ok(ObjectInfo::PitStartPoint(
                pit_start_point::PitStartPoint::from_wire(wire)?,
            )),
            186 => Ok(ObjectInfo::PitStopBox(pit::PitStopBox::from_wire(wire)?)),
            _ => Err(DecodeError::NoVariantMatch {
                found: index as u64,
            }),
        }
    }
}

impl ObjectInfo {
    /// Get heading if this object has one
    pub fn heading(&self) -> Option<crate::heading::Heading> {
        match self {
            ObjectInfo::Control(c) => Some(c.heading),
            ObjectInfo::Marshal(m) => Some(m.heading),
            ObjectInfo::InsimCheckpoint(ic) => Some(ic.heading),
            ObjectInfo::ChalkLine(c) => Some(c.heading),
            ObjectInfo::ChalkLine2(c) => Some(c.heading),
            ObjectInfo::ChalkAhead(c) => Some(c.heading),
            ObjectInfo::ChalkAhead2(c) => Some(c.heading),
            ObjectInfo::ChalkLeft(c) => Some(c.heading),
            ObjectInfo::ChalkLeft2(c) => Some(c.heading),
            ObjectInfo::ChalkLeft3(c) => Some(c.heading),
            ObjectInfo::ChalkRight(c) => Some(c.heading),
            ObjectInfo::ChalkRight2(c) => Some(c.heading),
            ObjectInfo::ChalkRight3(c) => Some(c.heading),
            ObjectInfo::PaintLetters(l) => Some(l.heading),
            ObjectInfo::PaintArrows(a) => Some(a.heading),
            ObjectInfo::Cone1(c) => Some(c.heading),
            ObjectInfo::Cone2(c) => Some(c.heading),
            ObjectInfo::ConeTall1(c) => Some(c.heading),
            ObjectInfo::ConeTall2(c) => Some(c.heading),
            ObjectInfo::ConePointer(cp) => Some(cp.heading),
            ObjectInfo::TyreSingle(t) => Some(t.heading),
            ObjectInfo::TyreStack2(t) => Some(t.heading),
            ObjectInfo::TyreStack3(t) => Some(t.heading),
            ObjectInfo::TyreStack4(t) => Some(t.heading),
            ObjectInfo::TyreSingleBig(t) => Some(t.heading),
            ObjectInfo::TyreStack2Big(t) => Some(t.heading),
            ObjectInfo::TyreStack3Big(t) => Some(t.heading),
            ObjectInfo::TyreStack4Big(t) => Some(t.heading),
            ObjectInfo::MarkerCorner(m) => Some(m.heading),
            ObjectInfo::MarkerDistance(m) => Some(m.heading),
            ObjectInfo::LetterboardWY(l) => Some(l.heading),
            ObjectInfo::LetterboardRB(l) => Some(l.heading),
            ObjectInfo::Armco1(a) => Some(a.heading),
            ObjectInfo::Armco3(a) => Some(a.heading),
            ObjectInfo::Armco5(a) => Some(a.heading),
            ObjectInfo::BarrierLong(b) => Some(b.heading),
            ObjectInfo::BarrierRed(b) => Some(b.heading),
            ObjectInfo::BarrierWhite(b) => Some(b.heading),
            ObjectInfo::Banner(b) => Some(b.heading),
            ObjectInfo::Ramp1(r) => Some(r.heading),
            ObjectInfo::Ramp2(r) => Some(r.heading),
            ObjectInfo::VehicleSUV(v) => Some(v.heading),
            ObjectInfo::VehicleVan(v) => Some(v.heading),
            ObjectInfo::VehicleTruck(v) => Some(v.heading),
            ObjectInfo::VehicleAmbulance(v) => Some(v.heading),
            ObjectInfo::SpeedHump10M(s) => Some(s.heading),
            ObjectInfo::SpeedHump6M(s) => Some(s.heading),
            ObjectInfo::SpeedHump2M(s) => Some(s.heading),
            ObjectInfo::SpeedHump1M(s) => Some(s.heading),
            ObjectInfo::Kerb(k) => Some(k.heading),
            ObjectInfo::Post(p) => Some(p.heading),
            ObjectInfo::Marquee(m) => Some(m.heading),
            ObjectInfo::Bale(b) => Some(b.heading),
            ObjectInfo::Bin1(b) => Some(b.heading),
            ObjectInfo::Bin2(b) => Some(b.heading),
            ObjectInfo::Railing1(r) => Some(r.heading),
            ObjectInfo::Railing2(r) => Some(r.heading),
            ObjectInfo::StartLights1(s) => Some(s.heading),
            ObjectInfo::StartLights2(s) => Some(s.heading),
            ObjectInfo::StartLights3(s) => Some(s.heading),
            ObjectInfo::SignMetal(s) => Some(s.heading),
            ObjectInfo::SignSpeed(s) => Some(s.heading),
            ObjectInfo::ConcreteSlab(c) => Some(c.heading),
            ObjectInfo::ConcreteRamp(c) => Some(c.heading),
            ObjectInfo::ConcreteWall(c) => Some(c.heading),
            ObjectInfo::ConcretePillar(c) => Some(c.heading),
            ObjectInfo::ConcreteSlabWall(c) => Some(c.heading),
            ObjectInfo::ConcreteRampWall(c) => Some(c.heading),
            ObjectInfo::ConcreteShortSlabWall(c) => Some(c.heading),
            ObjectInfo::ConcreteWedge(c) => Some(c.heading),
            ObjectInfo::StartPosition(s) => Some(s.heading),
            ObjectInfo::PitStartPoint(p) => Some(p.heading),
            ObjectInfo::PitStopBox(p) => Some(p.heading),
            ObjectInfo::InsimCircle(_) => None,
            ObjectInfo::RestrictedArea(_) => None,
            ObjectInfo::RouteChecker(_) => None,
            ObjectInfo::ChevronLeft(c) => Some(c.heading),
            ObjectInfo::ChevronRight(c) => Some(c.heading),
        }
    }

    /// Get floating flag if this object has one
    pub fn is_floating(&self) -> Option<bool> {
        match self {
            ObjectInfo::Control(c) => Some(c.floating),
            ObjectInfo::Marshal(m) => Some(m.floating),
            ObjectInfo::InsimCheckpoint(ic) => Some(ic.floating),
            ObjectInfo::InsimCircle(ic) => Some(ic.floating),
            ObjectInfo::RestrictedArea(ra) => Some(ra.floating),
            ObjectInfo::RouteChecker(rc) => Some(rc.floating),
            ObjectInfo::ChalkLine(c) => Some(c.floating),
            ObjectInfo::ChalkLine2(c) => Some(c.floating),
            ObjectInfo::ChalkAhead(c) => Some(c.floating),
            ObjectInfo::ChalkAhead2(c) => Some(c.floating),
            ObjectInfo::ChalkLeft(c) => Some(c.floating),
            ObjectInfo::ChalkLeft2(c) => Some(c.floating),
            ObjectInfo::ChalkLeft3(c) => Some(c.floating),
            ObjectInfo::ChalkRight(c) => Some(c.floating),
            ObjectInfo::ChalkRight2(c) => Some(c.floating),
            ObjectInfo::ChalkRight3(c) => Some(c.floating),
            ObjectInfo::PaintLetters(l) => Some(l.floating),
            ObjectInfo::PaintArrows(a) => Some(a.floating),
            ObjectInfo::Cone1(c) => Some(c.floating),
            ObjectInfo::Cone2(c) => Some(c.floating),
            ObjectInfo::ConeTall1(c) => Some(c.floating),
            ObjectInfo::ConeTall2(c) => Some(c.floating),
            ObjectInfo::ConePointer(cp) => Some(cp.floating),
            ObjectInfo::TyreSingle(t) => Some(t.floating),
            ObjectInfo::TyreStack2(t) => Some(t.floating),
            ObjectInfo::TyreStack3(t) => Some(t.floating),
            ObjectInfo::TyreStack4(t) => Some(t.floating),
            ObjectInfo::TyreSingleBig(t) => Some(t.floating),
            ObjectInfo::TyreStack2Big(t) => Some(t.floating),
            ObjectInfo::TyreStack3Big(t) => Some(t.floating),
            ObjectInfo::TyreStack4Big(t) => Some(t.floating),
            ObjectInfo::MarkerCorner(m) => Some(m.floating),
            ObjectInfo::MarkerDistance(m) => Some(m.floating),
            ObjectInfo::LetterboardWY(l) => Some(l.floating),
            ObjectInfo::LetterboardRB(l) => Some(l.floating),
            ObjectInfo::Armco1(a) => Some(a.floating),
            ObjectInfo::Armco3(a) => Some(a.floating),
            ObjectInfo::Armco5(a) => Some(a.floating),
            ObjectInfo::BarrierLong(b) => Some(b.floating),
            ObjectInfo::BarrierRed(b) => Some(b.floating),
            ObjectInfo::BarrierWhite(b) => Some(b.floating),
            ObjectInfo::Banner(b) => Some(b.floating),
            ObjectInfo::Ramp1(r) => Some(r.floating),
            ObjectInfo::Ramp2(r) => Some(r.floating),
            ObjectInfo::VehicleSUV(v) => Some(v.floating),
            ObjectInfo::VehicleVan(v) => Some(v.floating),
            ObjectInfo::VehicleTruck(v) => Some(v.floating),
            ObjectInfo::VehicleAmbulance(v) => Some(v.floating),
            ObjectInfo::SpeedHump10M(s) => Some(s.floating),
            ObjectInfo::SpeedHump6M(s) => Some(s.floating),
            ObjectInfo::SpeedHump2M(s) => Some(s.floating),
            ObjectInfo::SpeedHump1M(s) => Some(s.floating),
            ObjectInfo::Kerb(k) => Some(k.floating),
            ObjectInfo::Post(p) => Some(p.floating),
            ObjectInfo::Marquee(m) => Some(m.floating),
            ObjectInfo::Bale(b) => Some(b.floating),
            ObjectInfo::Bin1(b) => Some(b.floating),
            ObjectInfo::Bin2(b) => Some(b.floating),
            ObjectInfo::Railing1(r) => Some(r.floating),
            ObjectInfo::Railing2(r) => Some(r.floating),
            ObjectInfo::StartLights1(s) => Some(s.floating),
            ObjectInfo::StartLights2(s) => Some(s.floating),
            ObjectInfo::StartLights3(s) => Some(s.floating),
            ObjectInfo::SignMetal(s) => Some(s.floating),
            ObjectInfo::SignSpeed(s) => Some(s.floating),
            ObjectInfo::ConcreteSlab(_) => None, // Concrete objects always float (per spec)
            ObjectInfo::ConcreteRamp(_) => None,
            ObjectInfo::ConcreteWall(_) => None,
            ObjectInfo::ConcretePillar(_) => None,
            ObjectInfo::ConcreteSlabWall(_) => None,
            ObjectInfo::ConcreteRampWall(_) => None,
            ObjectInfo::ConcreteShortSlabWall(_) => None,
            ObjectInfo::ConcreteWedge(_) => None,
            ObjectInfo::StartPosition(s) => Some(s.floating),
            ObjectInfo::PitStartPoint(p) => Some(p.floating),
            ObjectInfo::PitStopBox(p) => Some(p.floating),
            ObjectInfo::ChevronLeft(p) => Some(p.floating),
            ObjectInfo::ChevronRight(p) => Some(p.floating),
        }
    }
}

impl Encode for ObjectInfo {
    fn encode(&self, buf: &mut bytes::BytesMut) -> Result<(), EncodeError> {
        let (index, wire) = match self {
            Self::Control(control) => {
                let wire = control.encode()?;
                (0, wire)
            },
            Self::Marshal(marshal) => {
                let wire = marshal.encode()?;
                (240, wire)
            },
            Self::InsimCheckpoint(insim_checkpoint) => {
                let wire = insim_checkpoint.encode()?;
                (252, wire)
            },
            Self::InsimCircle(insim_circle) => {
                let wire = insim_circle.encode()?;
                (253, wire)
            },
            Self::RestrictedArea(restricted_area) => {
                let wire = restricted_area.encode()?;
                (254, wire)
            },
            Self::RouteChecker(route_checker) => {
                let wire = route_checker.encode()?;
                (255, wire)
            },
            Self::ChalkLine(chalk) => {
                let wire = chalk.to_wire()?;
                (4, wire)
            },
            Self::ChalkLine2(chalk) => {
                let wire = chalk.to_wire()?;
                (5, wire)
            },
            Self::ChalkAhead(chalk) => {
                let wire = chalk.to_wire()?;
                (6, wire)
            },
            Self::ChalkAhead2(chalk) => {
                let wire = chalk.to_wire()?;
                (7, wire)
            },
            Self::ChalkLeft(chalk) => {
                let wire = chalk.to_wire()?;
                (8, wire)
            },
            Self::ChalkLeft2(chalk) => {
                let wire = chalk.to_wire()?;
                (9, wire)
            },
            Self::ChalkLeft3(chalk) => {
                let wire = chalk.to_wire()?;
                (10, wire)
            },
            Self::ChalkRight(chalk) => {
                let wire = chalk.to_wire()?;
                (11, wire)
            },
            Self::ChalkRight2(chalk) => {
                let wire = chalk.to_wire()?;
                (12, wire)
            },
            Self::ChalkRight3(chalk) => {
                let wire = chalk.to_wire()?;
                (13, wire)
            },
            Self::PaintLetters(letters) => {
                let wire = letters.to_wire()?;
                (16, wire)
            },
            Self::PaintArrows(arrows) => {
                let wire = arrows.to_wire()?;
                (17, wire)
            },
            Self::Cone1(cone1) => {
                let wire = cone1.to_wire()?;
                (20, wire)
            },
            Self::Cone2(cone2) => {
                let wire = cone2.to_wire()?;
                (21, wire)
            },
            Self::ConeTall1(cone_tall1) => {
                let wire = cone_tall1.to_wire()?;
                (32, wire)
            },
            Self::ConeTall2(cone_tall2) => {
                let wire = cone_tall2.to_wire()?;
                (33, wire)
            },
            Self::ConePointer(cone_pointer) => {
                let wire = cone_pointer.to_wire()?;
                (40, wire)
            },
            Self::TyreSingle(tyre) => {
                let wire = tyre.to_wire()?;
                (48, wire)
            },
            Self::TyreStack2(tyre) => {
                let wire = tyre.to_wire()?;
                (49, wire)
            },
            Self::TyreStack3(tyre) => {
                let wire = tyre.to_wire()?;
                (50, wire)
            },
            Self::TyreStack4(tyre) => {
                let wire = tyre.to_wire()?;
                (51, wire)
            },
            Self::TyreSingleBig(tyre) => {
                let wire = tyre.to_wire()?;
                (52, wire)
            },
            Self::TyreStack2Big(tyre) => {
                let wire = tyre.to_wire()?;
                (53, wire)
            },
            Self::TyreStack3Big(tyre) => {
                let wire = tyre.to_wire()?;
                (54, wire)
            },
            Self::TyreStack4Big(tyre) => {
                let wire = tyre.to_wire()?;
                (55, wire)
            },
            Self::MarkerCorner(marker_corner) => {
                let wire = marker_corner.to_wire()?;
                (62, wire)
            },
            Self::MarkerDistance(marker_distance) => {
                let wire = marker_distance.to_wire()?;
                (84, wire)
            },
            Self::LetterboardWY(letterboard_wy) => {
                let wire = letterboard_wy.to_wire()?;
                (92, wire)
            },
            Self::LetterboardRB(letterboard_rb) => {
                let wire = letterboard_rb.to_wire()?;
                (93, wire)
            },
            Self::Armco1(armco1) => {
                let wire = armco1.to_wire()?;
                (96, wire)
            },
            Self::Armco3(armco3) => {
                let wire = armco3.to_wire()?;
                (97, wire)
            },
            Self::Armco5(armco5) => {
                let wire = armco5.to_wire()?;
                (98, wire)
            },
            Self::BarrierLong(barrier) => {
                let wire = barrier.to_wire()?;
                (104, wire)
            },
            Self::BarrierRed(barrier) => {
                let wire = barrier.to_wire()?;
                (105, wire)
            },
            Self::BarrierWhite(barrier) => {
                let wire = barrier.to_wire()?;
                (106, wire)
            },
            Self::Banner(banner) => {
                let wire = banner.to_wire()?;
                (112, wire)
            },
            Self::Ramp1(ramp1) => {
                let wire = ramp1.to_wire()?;
                (120, wire)
            },
            Self::Ramp2(ramp2) => {
                let wire = ramp2.to_wire()?;
                (121, wire)
            },
            Self::VehicleSUV(veh) => {
                let wire = veh.to_wire()?;
                (124, wire)
            },
            Self::VehicleVan(veh) => {
                let wire = veh.to_wire()?;
                (125, wire)
            },
            Self::VehicleTruck(veh) => {
                let wire = veh.to_wire()?;
                (126, wire)
            },
            Self::VehicleAmbulance(veh) => {
                let wire = veh.to_wire()?;
                (127, wire)
            },
            Self::SpeedHump10M(speed_hump) => {
                let wire = speed_hump.to_wire()?;
                (128, wire)
            },
            Self::SpeedHump6M(speed_hump) => {
                let wire = speed_hump.to_wire()?;
                (129, wire)
            },
            Self::SpeedHump2M(speed_hump) => {
                let wire = speed_hump.to_wire()?;
                (130, wire)
            },
            Self::SpeedHump1M(speed_hump) => {
                let wire = speed_hump.to_wire()?;
                (131, wire)
            },
            Self::Kerb(kerb) => {
                let wire = kerb.to_wire()?;
                (132, wire)
            },
            Self::Post(post) => {
                let wire = post.to_wire()?;
                (136, wire)
            },
            Self::Marquee(marquee) => {
                let wire = marquee.to_wire()?;
                (140, wire)
            },
            Self::Bale(bale) => {
                let wire = bale.to_wire()?;
                (144, wire)
            },
            Self::Bin1(bin1) => {
                let wire = bin1.to_wire()?;
                (145, wire)
            },
            Self::Bin2(bin2) => {
                let wire = bin2.to_wire()?;
                (146, wire)
            },
            Self::Railing1(railing1) => {
                let wire = railing1.to_wire()?;
                (147, wire)
            },
            Self::Railing2(railing2) => {
                let wire = railing2.to_wire()?;
                (148, wire)
            },
            Self::StartLights1(start_lights1) => {
                let wire = start_lights1.to_wire()?;
                (149, wire)
            },
            Self::StartLights2(start_lights2) => {
                let wire = start_lights2.to_wire()?;
                (150, wire)
            },
            Self::StartLights3(start_lights3) => {
                let wire = start_lights3.to_wire()?;
                (151, wire)
            },
            Self::SignMetal(sign_metal) => {
                let wire = sign_metal.to_wire()?;
                (160, wire)
            },
            Self::ChevronLeft(chevron) => {
                let wire = chevron.to_wire()?;
                (164, wire)
            },
            Self::ChevronRight(chevron) => {
                let wire = chevron.to_wire()?;
                (165, wire)
            },
            Self::SignSpeed(sign_speed) => {
                let wire = sign_speed.to_wire()?;
                (168, wire)
            },
            Self::ConcreteSlab(concrete_slab) => {
                let wire = concrete_slab.to_wire()?;
                (172, wire)
            },
            Self::ConcreteRamp(concrete_ramp) => {
                let wire = concrete_ramp.to_wire()?;
                (173, wire)
            },
            Self::ConcreteWall(concrete_wall) => {
                let wire = concrete_wall.to_wire()?;
                (174, wire)
            },
            Self::ConcretePillar(concrete_pillar) => {
                let wire = concrete_pillar.to_wire()?;
                (175, wire)
            },
            Self::ConcreteSlabWall(concrete_slab_wall) => {
                let wire = concrete_slab_wall.to_wire()?;
                (176, wire)
            },
            Self::ConcreteRampWall(concrete_ramp_wall) => {
                let wire = concrete_ramp_wall.to_wire()?;
                (177, wire)
            },
            Self::ConcreteShortSlabWall(concrete_short_slab_wall) => {
                let wire = concrete_short_slab_wall.to_wire()?;
                (178, wire)
            },
            Self::ConcreteWedge(concrete_wedge) => {
                let wire = concrete_wedge.to_wire()?;
                (179, wire)
            },
            Self::StartPosition(start_position) => {
                let wire = start_position.to_wire()?;
                (184, wire)
            },
            Self::PitStartPoint(pit_start_point) => {
                let wire = pit_start_point.to_wire()?;
                (185, wire)
            },
            Self::PitStopBox(pit_stop_box) => {
                let wire = pit_stop_box.to_wire()?;
                (186, wire)
            },
        };

        wire.xyz.x.encode(buf)?;
        wire.xyz.y.encode(buf)?;
        wire.xyz.z.encode(buf)?;
        wire.flags.encode(buf)?;
        index.encode(buf)?;
        wire.heading.encode(buf)?;

        Ok(())
    }
}
