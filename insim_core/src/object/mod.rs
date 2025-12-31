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

use crate::{Decode, DecodeError, Encode, EncodeError, heading::Heading};

#[derive(Debug, Clone, Copy)]
pub(super) struct ObjectFlags(u8);
impl ObjectFlags {
    /// Check if the floating flag is set
    fn floating(&self) -> bool {
        self.0 & 0x80 != 0
    }

    /// Extract colour from flags (bits 0-2)
    fn colour(&self) -> u8 {
        self.0 & 0x07
    }

    /// Extract mapping from flags (bits 3-6)
    fn mapping(&self) -> u8 {
        (self.0 >> 3) & 0x0f
    }
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Layout Object Position
pub struct ObjectCoordinate {
    /// X coordinate (1:16 scale)
    pub x: i16,
    /// Y coordinate (1:16 scale)
    pub y: i16,
    /// X coordinate (1:4 scale)
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

        let flags = ObjectFlags(u8::decode(buf)?);
        let index = u8::decode(buf)?;
        let heading_u8 = u8::decode(buf)?;
        let heading = Heading::from_objectinfo_wire(heading_u8);

        match index {
            0 => Ok(ObjectInfo::Control(control::Control::new(
                xyz, flags, heading,
            )?)),
            240 => Ok(ObjectInfo::Marshal(marshal::Marshal::new(
                xyz, flags, heading,
            )?)),
            252 => Ok(ObjectInfo::InsimCheckpoint(insim::InsimCheckpoint::new(
                xyz, flags, heading,
            )?)),
            253 => Ok(ObjectInfo::InsimCircle(insim::InsimCircle::new(
                xyz, flags, heading_u8,
            )?)),
            254 => Ok(ObjectInfo::RestrictedArea(marshal::RestrictedArea::new(
                xyz, flags,
            )?)),
            255 => Ok(ObjectInfo::RouteChecker(marshal::RouteChecker::new(
                xyz, flags, heading_u8,
            )?)),

            4 => Ok(ObjectInfo::ChalkLine(chalk::Chalk::new(
                xyz, flags, heading,
            )?)),
            5 => Ok(ObjectInfo::ChalkLine2(chalk::Chalk::new(
                xyz, flags, heading,
            )?)),
            6 => Ok(ObjectInfo::ChalkAhead(chalk::Chalk::new(
                xyz, flags, heading,
            )?)),
            7 => Ok(ObjectInfo::ChalkAhead2(chalk::Chalk::new(
                xyz, flags, heading,
            )?)),
            8 => Ok(ObjectInfo::ChalkLeft(chalk::Chalk::new(
                xyz, flags, heading,
            )?)),
            9 => Ok(ObjectInfo::ChalkLeft2(chalk::Chalk::new(
                xyz, flags, heading,
            )?)),
            10 => Ok(ObjectInfo::ChalkLeft3(chalk::Chalk::new(
                xyz, flags, heading,
            )?)),
            11 => Ok(ObjectInfo::ChalkRight(chalk::Chalk::new(
                xyz, flags, heading,
            )?)),
            12 => Ok(ObjectInfo::ChalkRight2(chalk::Chalk::new(
                xyz, flags, heading,
            )?)),
            13 => Ok(ObjectInfo::ChalkRight3(chalk::Chalk::new(
                xyz, flags, heading,
            )?)),
            16 => Ok(ObjectInfo::PaintLetters(painted::Letters::new(
                xyz, flags, heading,
            )?)),
            17 => Ok(ObjectInfo::PaintArrows(painted::Arrows::new(
                xyz, flags, heading,
            )?)),
            20 => Ok(ObjectInfo::Cone1(cones::Cone::new(xyz, flags, heading)?)),
            21 => Ok(ObjectInfo::Cone2(cones::Cone::new(xyz, flags, heading)?)),
            32 => Ok(ObjectInfo::ConeTall1(cones::Cone::new(
                xyz, flags, heading,
            )?)),
            33 => Ok(ObjectInfo::ConeTall2(cones::Cone::new(
                xyz, flags, heading,
            )?)),
            40 => Ok(ObjectInfo::ConePointer(cones::Cone::new(
                xyz, flags, heading,
            )?)),

            48 => Ok(ObjectInfo::TyreSingle(tyres::Tyres::new(
                xyz, flags, heading,
            )?)),
            49 => Ok(ObjectInfo::TyreStack2(tyres::Tyres::new(
                xyz, flags, heading,
            )?)),
            50 => Ok(ObjectInfo::TyreStack3(tyres::Tyres::new(
                xyz, flags, heading,
            )?)),
            51 => Ok(ObjectInfo::TyreStack4(tyres::Tyres::new(
                xyz, flags, heading,
            )?)),
            52 => Ok(ObjectInfo::TyreSingleBig(tyres::Tyres::new(
                xyz, flags, heading,
            )?)),
            53 => Ok(ObjectInfo::TyreStack2Big(tyres::Tyres::new(
                xyz, flags, heading,
            )?)),
            54 => Ok(ObjectInfo::TyreStack3Big(tyres::Tyres::new(
                xyz, flags, heading,
            )?)),
            55 => Ok(ObjectInfo::TyreStack4Big(tyres::Tyres::new(
                xyz, flags, heading,
            )?)),

            62 => Ok(ObjectInfo::MarkerCorner(marker::MarkerCorner::new(
                xyz, flags, heading,
            )?)),
            84 => Ok(ObjectInfo::MarkerDistance(marker::MarkerDistance::new(
                xyz, flags, heading,
            )?)),
            92 => Ok(ObjectInfo::LetterboardWY(
                letterboard_wy::LetterboardWY::new(xyz, flags, heading)?,
            )),
            93 => Ok(ObjectInfo::LetterboardRB(
                letterboard_rb::LetterboardRB::new(xyz, flags, heading)?,
            )),
            96 => Ok(ObjectInfo::Armco1(armco::Armco::new(xyz, flags, heading)?)),
            97 => Ok(ObjectInfo::Armco3(armco::Armco::new(xyz, flags, heading)?)),
            98 => Ok(ObjectInfo::Armco5(armco::Armco::new(xyz, flags, heading)?)),
            104 => Ok(ObjectInfo::BarrierLong(barrier::Barrier::new(
                xyz, flags, heading,
            )?)),
            105 => Ok(ObjectInfo::BarrierRed(barrier::Barrier::new(
                xyz, flags, heading,
            )?)),
            106 => Ok(ObjectInfo::BarrierWhite(barrier::Barrier::new(
                xyz, flags, heading,
            )?)),
            112 => Ok(ObjectInfo::Banner(banner::Banner::new(
                xyz, flags, heading,
            )?)),
            120 => Ok(ObjectInfo::Ramp1(ramp::Ramp::new(xyz, flags, heading)?)),
            121 => Ok(ObjectInfo::Ramp2(ramp::Ramp::new(xyz, flags, heading)?)),
            124 => Ok(ObjectInfo::VehicleSUV(vehicle_suv::VehicleSUV::new(
                xyz, flags, heading,
            )?)),
            125 => Ok(ObjectInfo::VehicleVan(vehicle_van::VehicleVan::new(
                xyz, flags, heading,
            )?)),
            126 => Ok(ObjectInfo::VehicleTruck(vehicle_truck::VehicleTruck::new(
                xyz, flags, heading,
            )?)),
            127 => Ok(ObjectInfo::VehicleAmbulance(
                vehicle_ambulance::VehicleAmbulance::new(xyz, flags, heading)?,
            )),
            128 => Ok(ObjectInfo::SpeedHump10M(speed_hump::SpeedHump::new(
                xyz, flags, heading,
            )?)),
            129 => Ok(ObjectInfo::SpeedHump6M(speed_hump::SpeedHump::new(
                xyz, flags, heading,
            )?)),
            130 => Ok(ObjectInfo::SpeedHump2M(speed_hump::SpeedHump::new(
                xyz, flags, heading,
            )?)),
            131 => Ok(ObjectInfo::SpeedHump1M(speed_hump::SpeedHump::new(
                xyz, flags, heading,
            )?)),
            132 => Ok(ObjectInfo::Kerb(kerb::Kerb::new(xyz, flags, heading)?)),
            136 => Ok(ObjectInfo::Post(post::Post::new(xyz, flags, heading)?)),
            140 => Ok(ObjectInfo::Marquee(marquee::Marquee::new(
                xyz, flags, heading,
            )?)),
            144 => Ok(ObjectInfo::Bale(bale::Bale::new(xyz, flags, heading)?)),
            145 => Ok(ObjectInfo::Bin1(bin1::Bin1::new(xyz, flags, heading)?)),
            146 => Ok(ObjectInfo::Bin2(bin2::Bin2::new(xyz, flags, heading)?)),
            147 => Ok(ObjectInfo::Railing1(railing::Railing::new(
                xyz, flags, heading,
            )?)),
            148 => Ok(ObjectInfo::Railing2(railing::Railing::new(
                xyz, flags, heading,
            )?)),
            149 => Ok(ObjectInfo::StartLights1(start_lights::StartLights::new(
                xyz, flags, heading,
            )?)),
            150 => Ok(ObjectInfo::StartLights2(start_lights::StartLights::new(
                xyz, flags, heading,
            )?)),
            151 => Ok(ObjectInfo::StartLights3(start_lights::StartLights::new(
                xyz, flags, heading,
            )?)),
            160 => Ok(ObjectInfo::SignMetal(sign_metal::SignMetal::new(
                xyz, flags, heading,
            )?)),
            164 => Ok(ObjectInfo::ChevronLeft(chevron::Chevron::new(
                xyz, flags, heading,
            )?)),
            165 => Ok(ObjectInfo::ChevronRight(chevron::Chevron::new(
                xyz, flags, heading,
            )?)),
            168 => Ok(ObjectInfo::SignSpeed(sign_speed::SignSpeed::new(
                xyz, flags, heading,
            )?)),
            172 => Ok(ObjectInfo::ConcreteSlab(concrete::ConcreteSlab::new(
                xyz, flags, heading,
            )?)),
            173 => Ok(ObjectInfo::ConcreteRamp(concrete::ConcreteRamp::new(
                xyz, flags, heading,
            )?)),
            174 => Ok(ObjectInfo::ConcreteWall(concrete::ConcreteWall::new(
                xyz, flags, heading,
            )?)),
            175 => Ok(ObjectInfo::ConcretePillar(concrete::ConcretePillar::new(
                xyz, flags, heading,
            )?)),
            176 => Ok(ObjectInfo::ConcreteSlabWall(
                concrete::ConcreteSlabWall::new(xyz, flags, heading)?,
            )),
            177 => Ok(ObjectInfo::ConcreteRampWall(
                concrete::ConcreteRampWall::new(xyz, flags, heading)?,
            )),
            178 => Ok(ObjectInfo::ConcreteShortSlabWall(
                concrete::ConcreteShortSlabWall::new(xyz, flags, heading)?,
            )),
            179 => Ok(ObjectInfo::ConcreteWedge(concrete::ConcreteWedge::new(
                xyz, flags, heading,
            )?)),
            184 => Ok(ObjectInfo::StartPosition(
                start_position::StartPosition::new(xyz, flags, heading)?,
            )),
            185 => Ok(ObjectInfo::PitStartPoint(
                pit_start_point::PitStartPoint::new(xyz, flags, heading)?,
            )),
            186 => Ok(ObjectInfo::PitStopBox(pit::PitStopBox::new(
                xyz, flags, heading,
            )?)),
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

    /// Raw XYZ position
    pub fn position(&self) -> ObjectCoordinate {
        match self {
            ObjectInfo::Control(i) => i.xyz,
            ObjectInfo::Marshal(i) => i.xyz,
            ObjectInfo::InsimCheckpoint(i) => i.xyz,
            ObjectInfo::InsimCircle(i) => i.xyz,
            ObjectInfo::RestrictedArea(i) => i.xyz,
            ObjectInfo::RouteChecker(i) => i.xyz,
            ObjectInfo::ChalkLine(i) => i.xyz,
            ObjectInfo::ChalkLine2(i) => i.xyz,
            ObjectInfo::ChalkAhead(i) => i.xyz,
            ObjectInfo::ChalkAhead2(i) => i.xyz,
            ObjectInfo::ChalkLeft(i) => i.xyz,
            ObjectInfo::ChalkLeft2(i) => i.xyz,
            ObjectInfo::ChalkLeft3(i) => i.xyz,
            ObjectInfo::ChalkRight(i) => i.xyz,
            ObjectInfo::ChalkRight2(i) => i.xyz,
            ObjectInfo::ChalkRight3(i) => i.xyz,
            ObjectInfo::PaintLetters(i) => i.xyz,
            ObjectInfo::PaintArrows(i) => i.xyz,
            ObjectInfo::Cone1(i) => i.xyz,
            ObjectInfo::Cone2(i) => i.xyz,
            ObjectInfo::ConeTall1(i) => i.xyz,
            ObjectInfo::ConeTall2(i) => i.xyz,
            ObjectInfo::ConePointer(i) => i.xyz,
            ObjectInfo::TyreSingle(i) => i.xyz,
            ObjectInfo::TyreStack2(i) => i.xyz,
            ObjectInfo::TyreStack3(i) => i.xyz,
            ObjectInfo::TyreStack4(i) => i.xyz,
            ObjectInfo::TyreSingleBig(i) => i.xyz,
            ObjectInfo::TyreStack2Big(i) => i.xyz,
            ObjectInfo::TyreStack3Big(i) => i.xyz,
            ObjectInfo::TyreStack4Big(i) => i.xyz,
            ObjectInfo::MarkerCorner(i) => i.xyz,
            ObjectInfo::MarkerDistance(i) => i.xyz,
            ObjectInfo::LetterboardWY(i) => i.xyz,
            ObjectInfo::LetterboardRB(i) => i.xyz,
            ObjectInfo::Armco1(i) => i.xyz,
            ObjectInfo::Armco3(i) => i.xyz,
            ObjectInfo::Armco5(i) => i.xyz,
            ObjectInfo::BarrierLong(i) => i.xyz,
            ObjectInfo::BarrierRed(i) => i.xyz,
            ObjectInfo::BarrierWhite(i) => i.xyz,
            ObjectInfo::Banner(i) => i.xyz,
            ObjectInfo::Ramp1(i) => i.xyz,
            ObjectInfo::Ramp2(i) => i.xyz,
            ObjectInfo::VehicleSUV(i) => i.xyz,
            ObjectInfo::VehicleVan(i) => i.xyz,
            ObjectInfo::VehicleTruck(i) => i.xyz,
            ObjectInfo::VehicleAmbulance(i) => i.xyz,
            ObjectInfo::Kerb(i) => i.xyz,
            ObjectInfo::Post(i) => i.xyz,
            ObjectInfo::Marquee(i) => i.xyz,
            ObjectInfo::Bale(i) => i.xyz,
            ObjectInfo::SpeedHump10M(i) => i.xyz,
            ObjectInfo::SpeedHump6M(i) => i.xyz,
            ObjectInfo::SpeedHump2M(i) => i.xyz,
            ObjectInfo::SpeedHump1M(i) => i.xyz,
            ObjectInfo::Bin1(i) => i.xyz,
            ObjectInfo::Bin2(i) => i.xyz,
            ObjectInfo::Railing1(i) => i.xyz,
            ObjectInfo::Railing2(i) => i.xyz,
            ObjectInfo::StartLights1(i) => i.xyz,
            ObjectInfo::StartLights2(i) => i.xyz,
            ObjectInfo::StartLights3(i) => i.xyz,
            ObjectInfo::SignMetal(i) => i.xyz,
            ObjectInfo::ChevronLeft(i) => i.xyz,
            ObjectInfo::ChevronRight(i) => i.xyz,
            ObjectInfo::SignSpeed(i) => i.xyz,
            ObjectInfo::ConcreteSlab(i) => i.xyz,
            ObjectInfo::ConcreteRamp(i) => i.xyz,
            ObjectInfo::ConcreteWall(i) => i.xyz,
            ObjectInfo::ConcretePillar(i) => i.xyz,
            ObjectInfo::ConcreteSlabWall(i) => i.xyz,
            ObjectInfo::ConcreteRampWall(i) => i.xyz,
            ObjectInfo::ConcreteShortSlabWall(i) => i.xyz,
            ObjectInfo::ConcreteWedge(i) => i.xyz,
            ObjectInfo::StartPosition(i) => i.xyz,
            ObjectInfo::PitStartPoint(i) => i.xyz,
            ObjectInfo::PitStopBox(i) => i.xyz,
        }
    }

    /// Mutable raw XYZ position
    pub fn position_mut(&mut self) -> &mut ObjectCoordinate {
        match self {
            ObjectInfo::Control(i) => &mut i.xyz,
            ObjectInfo::Marshal(i) => &mut i.xyz,
            ObjectInfo::InsimCheckpoint(i) => &mut i.xyz,
            ObjectInfo::InsimCircle(i) => &mut i.xyz,
            ObjectInfo::RestrictedArea(i) => &mut i.xyz,
            ObjectInfo::RouteChecker(i) => &mut i.xyz,
            ObjectInfo::ChalkLine(i) => &mut i.xyz,
            ObjectInfo::ChalkLine2(i) => &mut i.xyz,
            ObjectInfo::ChalkAhead(i) => &mut i.xyz,
            ObjectInfo::ChalkAhead2(i) => &mut i.xyz,
            ObjectInfo::ChalkLeft(i) => &mut i.xyz,
            ObjectInfo::ChalkLeft2(i) => &mut i.xyz,
            ObjectInfo::ChalkLeft3(i) => &mut i.xyz,
            ObjectInfo::ChalkRight(i) => &mut i.xyz,
            ObjectInfo::ChalkRight2(i) => &mut i.xyz,
            ObjectInfo::ChalkRight3(i) => &mut i.xyz,
            ObjectInfo::PaintLetters(i) => &mut i.xyz,
            ObjectInfo::PaintArrows(i) => &mut i.xyz,
            ObjectInfo::Cone1(i) => &mut i.xyz,
            ObjectInfo::Cone2(i) => &mut i.xyz,
            ObjectInfo::ConeTall1(i) => &mut i.xyz,
            ObjectInfo::ConeTall2(i) => &mut i.xyz,
            ObjectInfo::ConePointer(i) => &mut i.xyz,
            ObjectInfo::TyreSingle(i) => &mut i.xyz,
            ObjectInfo::TyreStack2(i) => &mut i.xyz,
            ObjectInfo::TyreStack3(i) => &mut i.xyz,
            ObjectInfo::TyreStack4(i) => &mut i.xyz,
            ObjectInfo::TyreSingleBig(i) => &mut i.xyz,
            ObjectInfo::TyreStack2Big(i) => &mut i.xyz,
            ObjectInfo::TyreStack3Big(i) => &mut i.xyz,
            ObjectInfo::TyreStack4Big(i) => &mut i.xyz,
            ObjectInfo::MarkerCorner(i) => &mut i.xyz,
            ObjectInfo::MarkerDistance(i) => &mut i.xyz,
            ObjectInfo::LetterboardWY(i) => &mut i.xyz,
            ObjectInfo::LetterboardRB(i) => &mut i.xyz,
            ObjectInfo::Armco1(i) => &mut i.xyz,
            ObjectInfo::Armco3(i) => &mut i.xyz,
            ObjectInfo::Armco5(i) => &mut i.xyz,
            ObjectInfo::BarrierLong(i) => &mut i.xyz,
            ObjectInfo::BarrierRed(i) => &mut i.xyz,
            ObjectInfo::BarrierWhite(i) => &mut i.xyz,
            ObjectInfo::Banner(i) => &mut i.xyz,
            ObjectInfo::Ramp1(i) => &mut i.xyz,
            ObjectInfo::Ramp2(i) => &mut i.xyz,
            ObjectInfo::VehicleSUV(i) => &mut i.xyz,
            ObjectInfo::VehicleVan(i) => &mut i.xyz,
            ObjectInfo::VehicleTruck(i) => &mut i.xyz,
            ObjectInfo::VehicleAmbulance(i) => &mut i.xyz,
            ObjectInfo::Kerb(i) => &mut i.xyz,
            ObjectInfo::Post(i) => &mut i.xyz,
            ObjectInfo::Marquee(i) => &mut i.xyz,
            ObjectInfo::Bale(i) => &mut i.xyz,
            ObjectInfo::SpeedHump10M(i) => &mut i.xyz,
            ObjectInfo::SpeedHump6M(i) => &mut i.xyz,
            ObjectInfo::SpeedHump2M(i) => &mut i.xyz,
            ObjectInfo::SpeedHump1M(i) => &mut i.xyz,
            ObjectInfo::Bin1(i) => &mut i.xyz,
            ObjectInfo::Bin2(i) => &mut i.xyz,
            ObjectInfo::Railing1(i) => &mut i.xyz,
            ObjectInfo::Railing2(i) => &mut i.xyz,
            ObjectInfo::StartLights1(i) => &mut i.xyz,
            ObjectInfo::StartLights2(i) => &mut i.xyz,
            ObjectInfo::StartLights3(i) => &mut i.xyz,
            ObjectInfo::SignMetal(i) => &mut i.xyz,
            ObjectInfo::ChevronLeft(i) => &mut i.xyz,
            ObjectInfo::ChevronRight(i) => &mut i.xyz,
            ObjectInfo::SignSpeed(i) => &mut i.xyz,
            ObjectInfo::ConcreteSlab(i) => &mut i.xyz,
            ObjectInfo::ConcreteRamp(i) => &mut i.xyz,
            ObjectInfo::ConcreteWall(i) => &mut i.xyz,
            ObjectInfo::ConcretePillar(i) => &mut i.xyz,
            ObjectInfo::ConcreteSlabWall(i) => &mut i.xyz,
            ObjectInfo::ConcreteRampWall(i) => &mut i.xyz,
            ObjectInfo::ConcreteShortSlabWall(i) => &mut i.xyz,
            ObjectInfo::ConcreteWedge(i) => &mut i.xyz,
            ObjectInfo::StartPosition(i) => &mut i.xyz,
            ObjectInfo::PitStartPoint(i) => &mut i.xyz,
            ObjectInfo::PitStopBox(i) => &mut i.xyz,
        }
    }
}

impl Encode for ObjectInfo {
    fn encode(&self, buf: &mut bytes::BytesMut) -> Result<(), EncodeError> {
        let (index, xyz, flags, heading): (u8, ObjectCoordinate, ObjectFlags, u8) = match self {
            Self::Control(control) => (
                0,
                control.xyz,
                control.to_flags(),
                control.heading.to_objectinfo_wire(),
            ),
            Self::Marshal(marshal) => (
                240,
                marshal.xyz,
                marshal.to_flags(),
                marshal.heading.to_objectinfo_wire(),
            ),
            Self::InsimCheckpoint(insim_checkpoint) => (
                252,
                insim_checkpoint.xyz,
                insim_checkpoint.to_flags(),
                insim_checkpoint.heading.to_objectinfo_wire(),
            ),
            Self::InsimCircle(insim_circle) => (
                253,
                insim_circle.xyz,
                insim_circle.to_flags(),
                insim_circle.index,
            ),
            Self::RestrictedArea(restricted_area) => {
                (254, restricted_area.xyz, restricted_area.to_flags(), 0)
            },
            Self::RouteChecker(route_checker) => {
                (255, route_checker.xyz, route_checker.to_flags(), 0)
            },
            Self::ChalkLine(chalk) => (
                4,
                chalk.xyz,
                chalk.to_flags(),
                chalk.heading.to_objectinfo_wire(),
            ),
            Self::ChalkLine2(chalk) => (
                5,
                chalk.xyz,
                chalk.to_flags(),
                chalk.heading.to_objectinfo_wire(),
            ),
            Self::ChalkAhead(chalk) => (
                6,
                chalk.xyz,
                chalk.to_flags(),
                chalk.heading.to_objectinfo_wire(),
            ),
            Self::ChalkAhead2(chalk) => (
                7,
                chalk.xyz,
                chalk.to_flags(),
                chalk.heading.to_objectinfo_wire(),
            ),
            Self::ChalkLeft(chalk) => (
                8,
                chalk.xyz,
                chalk.to_flags(),
                chalk.heading.to_objectinfo_wire(),
            ),
            Self::ChalkLeft2(chalk) => (
                9,
                chalk.xyz,
                chalk.to_flags(),
                chalk.heading.to_objectinfo_wire(),
            ),
            Self::ChalkLeft3(chalk) => (
                10,
                chalk.xyz,
                chalk.to_flags(),
                chalk.heading.to_objectinfo_wire(),
            ),
            Self::ChalkRight(chalk) => (
                11,
                chalk.xyz,
                chalk.to_flags(),
                chalk.heading.to_objectinfo_wire(),
            ),
            Self::ChalkRight2(chalk) => (
                12,
                chalk.xyz,
                chalk.to_flags(),
                chalk.heading.to_objectinfo_wire(),
            ),
            Self::ChalkRight3(chalk) => (
                13,
                chalk.xyz,
                chalk.to_flags(),
                chalk.heading.to_objectinfo_wire(),
            ),
            Self::PaintLetters(letters) => (
                16,
                letters.xyz,
                letters.to_flags(),
                letters.heading.to_objectinfo_wire(),
            ),
            Self::PaintArrows(arrows) => (
                17,
                arrows.xyz,
                arrows.to_flags(),
                arrows.heading.to_objectinfo_wire(),
            ),
            Self::Cone1(cone) => (
                20,
                cone.xyz,
                cone.to_flags(),
                cone.heading.to_objectinfo_wire(),
            ),
            Self::Cone2(cone) => (
                21,
                cone.xyz,
                cone.to_flags(),
                cone.heading.to_objectinfo_wire(),
            ),
            Self::ConeTall1(cone) => (
                32,
                cone.xyz,
                cone.to_flags(),
                cone.heading.to_objectinfo_wire(),
            ),
            Self::ConeTall2(cone) => (
                33,
                cone.xyz,
                cone.to_flags(),
                cone.heading.to_objectinfo_wire(),
            ),
            Self::ConePointer(cone) => (
                40,
                cone.xyz,
                cone.to_flags(),
                cone.heading.to_objectinfo_wire(),
            ),
            Self::TyreSingle(tyre) => (
                48,
                tyre.xyz,
                tyre.to_flags(),
                tyre.heading.to_objectinfo_wire(),
            ),
            Self::TyreStack2(tyre) => (
                49,
                tyre.xyz,
                tyre.to_flags(),
                tyre.heading.to_objectinfo_wire(),
            ),
            Self::TyreStack3(tyre) => (
                50,
                tyre.xyz,
                tyre.to_flags(),
                tyre.heading.to_objectinfo_wire(),
            ),
            Self::TyreStack4(tyre) => (
                51,
                tyre.xyz,
                tyre.to_flags(),
                tyre.heading.to_objectinfo_wire(),
            ),
            Self::TyreSingleBig(tyre) => (
                52,
                tyre.xyz,
                tyre.to_flags(),
                tyre.heading.to_objectinfo_wire(),
            ),
            Self::TyreStack2Big(tyre) => (
                53,
                tyre.xyz,
                tyre.to_flags(),
                tyre.heading.to_objectinfo_wire(),
            ),
            Self::TyreStack3Big(tyre) => (
                54,
                tyre.xyz,
                tyre.to_flags(),
                tyre.heading.to_objectinfo_wire(),
            ),
            Self::TyreStack4Big(tyre) => (
                55,
                tyre.xyz,
                tyre.to_flags(),
                tyre.heading.to_objectinfo_wire(),
            ),
            Self::MarkerCorner(marker_corner) => (
                62,
                marker_corner.xyz,
                marker_corner.to_flags(),
                marker_corner.heading.to_objectinfo_wire(),
            ),
            Self::MarkerDistance(marker_distance) => (
                84,
                marker_distance.xyz,
                marker_distance.to_flags(),
                marker_distance.heading.to_objectinfo_wire(),
            ),
            Self::LetterboardWY(letterboard_wy) => (
                92,
                letterboard_wy.xyz,
                letterboard_wy.to_flags(),
                letterboard_wy.heading.to_objectinfo_wire(),
            ),
            Self::LetterboardRB(letterboard_rb) => (
                93,
                letterboard_rb.xyz,
                letterboard_rb.to_flags(),
                letterboard_rb.heading.to_objectinfo_wire(),
            ),
            Self::Armco1(armco1) => {
                let flags = armco1.to_flags();
                (96, armco1.xyz, flags, armco1.heading.to_objectinfo_wire())
            },
            Self::Armco3(armco3) => {
                let flags = armco3.to_flags();
                (97, armco3.xyz, flags, armco3.heading.to_objectinfo_wire())
            },
            Self::Armco5(armco5) => {
                let flags = armco5.to_flags();
                (98, armco5.xyz, flags, armco5.heading.to_objectinfo_wire())
            },
            Self::BarrierLong(barrier) => (
                104,
                barrier.xyz,
                barrier.to_flags(),
                barrier.heading.to_objectinfo_wire(),
            ),
            Self::BarrierRed(barrier) => (
                105,
                barrier.xyz,
                barrier.to_flags(),
                barrier.heading.to_objectinfo_wire(),
            ),
            Self::BarrierWhite(barrier) => (
                106,
                barrier.xyz,
                barrier.to_flags(),
                barrier.heading.to_objectinfo_wire(),
            ),
            Self::Banner(banner) => {
                let wire = banner.to_flags();
                (112, banner.xyz, wire, banner.heading.to_objectinfo_wire())
            },
            Self::Ramp1(ramp) => (
                120,
                ramp.xyz,
                ramp.to_flags(),
                ramp.heading.to_objectinfo_wire(),
            ),
            Self::Ramp2(ramp) => (
                121,
                ramp.xyz,
                ramp.to_flags(),
                ramp.heading.to_objectinfo_wire(),
            ),
            Self::VehicleSUV(veh) => (
                124,
                veh.xyz,
                veh.to_flags(),
                veh.heading.to_objectinfo_wire(),
            ),
            Self::VehicleVan(veh) => (
                125,
                veh.xyz,
                veh.to_flags(),
                veh.heading.to_objectinfo_wire(),
            ),
            Self::VehicleTruck(veh) => (
                126,
                veh.xyz,
                veh.to_flags(),
                veh.heading.to_objectinfo_wire(),
            ),
            Self::VehicleAmbulance(veh) => (
                127,
                veh.xyz,
                veh.to_flags(),
                veh.heading.to_objectinfo_wire(),
            ),
            Self::SpeedHump10M(speed_hump) => (
                128,
                speed_hump.xyz,
                speed_hump.to_flags(),
                speed_hump.heading.to_objectinfo_wire(),
            ),
            Self::SpeedHump6M(speed_hump) => (
                129,
                speed_hump.xyz,
                speed_hump.to_flags(),
                speed_hump.heading.to_objectinfo_wire(),
            ),
            Self::SpeedHump2M(speed_hump) => (
                130,
                speed_hump.xyz,
                speed_hump.to_flags(),
                speed_hump.heading.to_objectinfo_wire(),
            ),
            Self::SpeedHump1M(speed_hump) => (
                131,
                speed_hump.xyz,
                speed_hump.to_flags(),
                speed_hump.heading.to_objectinfo_wire(),
            ),
            Self::Kerb(kerb) => (
                132,
                kerb.xyz,
                kerb.to_flags(),
                kerb.heading.to_objectinfo_wire(),
            ),
            Self::Post(post) => (
                136,
                post.xyz,
                post.to_flags(),
                post.heading.to_objectinfo_wire(),
            ),
            Self::Marquee(marquee) => (
                140,
                marquee.xyz,
                marquee.to_flags(),
                marquee.heading.to_objectinfo_wire(),
            ),
            Self::Bale(bale) => {
                let flags = bale.to_flags();
                (144, bale.xyz, flags, bale.heading.to_objectinfo_wire())
            },
            Self::Bin1(bin1) => (
                145,
                bin1.xyz,
                bin1.to_flags(),
                bin1.heading.to_objectinfo_wire(),
            ),
            Self::Bin2(bin2) => (
                146,
                bin2.xyz,
                bin2.to_flags(),
                bin2.heading.to_objectinfo_wire(),
            ),
            Self::Railing1(railing) => (
                147,
                railing.xyz,
                railing.to_flags(),
                railing.heading.to_objectinfo_wire(),
            ),
            Self::Railing2(railing) => (
                148,
                railing.xyz,
                railing.to_flags(),
                railing.heading.to_objectinfo_wire(),
            ),
            Self::StartLights1(start_lights) => (
                149,
                start_lights.xyz,
                start_lights.to_flags(),
                start_lights.heading.to_objectinfo_wire(),
            ),
            Self::StartLights2(start_lights) => (
                150,
                start_lights.xyz,
                start_lights.to_flags(),
                start_lights.heading.to_objectinfo_wire(),
            ),
            Self::StartLights3(start_lights) => (
                151,
                start_lights.xyz,
                start_lights.to_flags(),
                start_lights.heading.to_objectinfo_wire(),
            ),
            Self::SignMetal(sign_metal) => (
                160,
                sign_metal.xyz,
                sign_metal.to_flags(),
                sign_metal.heading.to_objectinfo_wire(),
            ),
            Self::ChevronLeft(chevron) => (
                164,
                chevron.xyz,
                chevron.to_flags(),
                chevron.heading.to_objectinfo_wire(),
            ),
            Self::ChevronRight(chevron) => (
                165,
                chevron.xyz,
                chevron.to_flags(),
                chevron.heading.to_objectinfo_wire(),
            ),
            Self::SignSpeed(sign_speed) => (
                168,
                sign_speed.xyz,
                sign_speed.to_flags(),
                sign_speed.heading.to_objectinfo_wire(),
            ),
            Self::ConcreteSlab(concrete) => (
                172,
                concrete.xyz,
                concrete.to_flags(),
                concrete.heading.to_objectinfo_wire(),
            ),
            Self::ConcreteRamp(concrete) => (
                173,
                concrete.xyz,
                concrete.to_flags(),
                concrete.heading.to_objectinfo_wire(),
            ),
            Self::ConcreteWall(concrete) => (
                174,
                concrete.xyz,
                concrete.to_flags(),
                concrete.heading.to_objectinfo_wire(),
            ),
            Self::ConcretePillar(concrete) => (
                175,
                concrete.xyz,
                concrete.to_flags(),
                concrete.heading.to_objectinfo_wire(),
            ),
            Self::ConcreteSlabWall(concrete) => (
                176,
                concrete.xyz,
                concrete.to_flags(),
                concrete.heading.to_objectinfo_wire(),
            ),
            Self::ConcreteRampWall(concrete) => (
                177,
                concrete.xyz,
                concrete.to_flags(),
                concrete.heading.to_objectinfo_wire(),
            ),
            Self::ConcreteShortSlabWall(concrete) => (
                178,
                concrete.xyz,
                concrete.to_flags(),
                concrete.heading.to_objectinfo_wire(),
            ),
            Self::ConcreteWedge(concrete) => (
                179,
                concrete.xyz,
                concrete.to_flags(),
                concrete.heading.to_objectinfo_wire(),
            ),
            Self::StartPosition(start_position) => (
                184,
                start_position.xyz,
                start_position.to_flags(),
                start_position.heading.to_objectinfo_wire(),
            ),
            Self::PitStartPoint(pit_start_point) => (
                185,
                pit_start_point.xyz,
                pit_start_point.to_flags(),
                pit_start_point.heading.to_objectinfo_wire(),
            ),
            Self::PitStopBox(pit_stop_box) => (
                186,
                pit_stop_box.xyz,
                pit_stop_box.to_flags(),
                pit_stop_box.heading.to_objectinfo_wire(),
            ),
        };

        xyz.x.encode(buf)?;
        xyz.y.encode(buf)?;
        xyz.z.encode(buf)?;
        flags.0.encode(buf)?;
        index.encode(buf)?;
        heading.encode(buf)?;

        Ok(())
    }
}
