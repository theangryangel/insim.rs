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

pub mod object_coordinate;
mod object_flags;
#[cfg(test)]
mod tests;

pub use object_coordinate::ObjectCoordinate;
use object_flags::ObjectFlags;

use crate::{Decode, DecodeError, DecodeErrorKind, Encode, EncodeError, heading::Heading};

// TODO: We could probably DRY this up with a proc macro to make life a lot easier now that we're
// happy with this. However, I have no desire to do this right now. I'd rather build something.

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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

impl From<&ObjectInfo> for u8 {
    fn from(value: &ObjectInfo) -> Self {
        match value {
            ObjectInfo::Control(_) => 0,
            ObjectInfo::Marshal(_) => 240,
            ObjectInfo::InsimCheckpoint(_) => 252,
            ObjectInfo::InsimCircle(_) => 253,
            ObjectInfo::RestrictedArea(_) => 254,
            ObjectInfo::RouteChecker(_) => 255,
            ObjectInfo::ChalkLine(_) => 4,
            ObjectInfo::ChalkLine2(_) => 5,
            ObjectInfo::ChalkAhead(_) => 6,
            ObjectInfo::ChalkAhead2(_) => 7,
            ObjectInfo::ChalkLeft(_) => 8,
            ObjectInfo::ChalkLeft2(_) => 9,
            ObjectInfo::ChalkLeft3(_) => 10,
            ObjectInfo::ChalkRight(_) => 11,
            ObjectInfo::ChalkRight2(_) => 12,
            ObjectInfo::ChalkRight3(_) => 13,
            ObjectInfo::PaintLetters(_) => 16,
            ObjectInfo::PaintArrows(_) => 17,
            ObjectInfo::Cone1(_) => 20,
            ObjectInfo::Cone2(_) => 21,
            ObjectInfo::ConeTall1(_) => 32,
            ObjectInfo::ConeTall2(_) => 33,
            ObjectInfo::ConePointer(_) => 40,
            ObjectInfo::TyreSingle(_) => 48,
            ObjectInfo::TyreStack2(_) => 49,
            ObjectInfo::TyreStack3(_) => 50,
            ObjectInfo::TyreStack4(_) => 51,
            ObjectInfo::TyreSingleBig(_) => 52,
            ObjectInfo::TyreStack2Big(_) => 53,
            ObjectInfo::TyreStack3Big(_) => 54,
            ObjectInfo::TyreStack4Big(_) => 55,
            ObjectInfo::MarkerCorner(_) => 64,
            ObjectInfo::MarkerDistance(_) => 84,
            ObjectInfo::LetterboardWY(_) => 92,
            ObjectInfo::LetterboardRB(_) => 93,
            ObjectInfo::Armco1(_) => 96,
            ObjectInfo::Armco3(_) => 97,
            ObjectInfo::Armco5(_) => 98,
            ObjectInfo::BarrierLong(_) => 104,
            ObjectInfo::BarrierRed(_) => 105,
            ObjectInfo::BarrierWhite(_) => 106,
            ObjectInfo::Banner(_) => 112,
            ObjectInfo::Ramp1(_) => 120,
            ObjectInfo::Ramp2(_) => 121,
            ObjectInfo::VehicleSUV(_) => 124,
            ObjectInfo::VehicleVan(_) => 125,
            ObjectInfo::VehicleTruck(_) => 126,
            ObjectInfo::VehicleAmbulance(_) => 127,
            ObjectInfo::SpeedHump10M(_) => 128,
            ObjectInfo::SpeedHump6M(_) => 129,
            ObjectInfo::SpeedHump2M(_) => 130,
            ObjectInfo::SpeedHump1M(_) => 131,
            ObjectInfo::Kerb(_) => 132,
            ObjectInfo::Post(_) => 136,
            ObjectInfo::Marquee(_) => 140,
            ObjectInfo::Bale(_) => 144,
            ObjectInfo::Bin1(_) => 145,
            ObjectInfo::Bin2(_) => 146,
            ObjectInfo::Railing1(_) => 147,
            ObjectInfo::Railing2(_) => 148,
            ObjectInfo::StartLights1(_) => 149,
            ObjectInfo::StartLights2(_) => 150,
            ObjectInfo::StartLights3(_) => 151,
            ObjectInfo::SignMetal(_) => 160,
            ObjectInfo::ChevronLeft(_) => 164,
            ObjectInfo::ChevronRight(_) => 165,
            ObjectInfo::SignSpeed(_) => 168,
            ObjectInfo::ConcreteSlab(_) => 172,
            ObjectInfo::ConcreteRamp(_) => 173,
            ObjectInfo::ConcreteWall(_) => 174,
            ObjectInfo::ConcretePillar(_) => 175,
            ObjectInfo::ConcreteSlabWall(_) => 176,
            ObjectInfo::ConcreteRampWall(_) => 177,
            ObjectInfo::ConcreteShortSlabWall(_) => 178,
            ObjectInfo::ConcreteWedge(_) => 179,
            ObjectInfo::StartPosition(_) => 184,
            ObjectInfo::PitStartPoint(_) => 185,
            ObjectInfo::PitStopBox(_) => 186,
        }
    }
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

            64 => Ok(ObjectInfo::MarkerCorner(marker::MarkerCorner::new(
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
            _ => Err(DecodeErrorKind::NoVariantMatch {
                found: index as u64,
            }
            .into()),
        }
    }
}

impl ObjectInfo {
    /// Set heading if this object has one
    pub fn set_heading(&mut self, heading: Heading) -> bool {
        match self {
            ObjectInfo::Control(o) => o.heading = heading,
            ObjectInfo::Marshal(o) => o.heading = heading,
            ObjectInfo::InsimCheckpoint(o) => o.heading = heading,
            ObjectInfo::ChalkLine(o) => o.heading = heading,
            ObjectInfo::ChalkLine2(o) => o.heading = heading,
            ObjectInfo::ChalkAhead(o) => o.heading = heading,
            ObjectInfo::ChalkAhead2(o) => o.heading = heading,
            ObjectInfo::ChalkLeft(o) => o.heading = heading,
            ObjectInfo::ChalkLeft2(o) => o.heading = heading,
            ObjectInfo::ChalkLeft3(o) => o.heading = heading,
            ObjectInfo::ChalkRight(o) => o.heading = heading,
            ObjectInfo::ChalkRight2(o) => o.heading = heading,
            ObjectInfo::ChalkRight3(o) => o.heading = heading,
            ObjectInfo::PaintLetters(o) => o.heading = heading,
            ObjectInfo::PaintArrows(o) => o.heading = heading,
            ObjectInfo::Cone1(o) => o.heading = heading,
            ObjectInfo::Cone2(o) => o.heading = heading,
            ObjectInfo::ConeTall1(o) => o.heading = heading,
            ObjectInfo::ConeTall2(o) => o.heading = heading,
            ObjectInfo::ConePointer(o) => o.heading = heading,
            ObjectInfo::TyreSingle(o) => o.heading = heading,
            ObjectInfo::TyreStack2(o) => o.heading = heading,
            ObjectInfo::TyreStack3(o) => o.heading = heading,
            ObjectInfo::TyreStack4(o) => o.heading = heading,
            ObjectInfo::TyreSingleBig(o) => o.heading = heading,
            ObjectInfo::TyreStack2Big(o) => o.heading = heading,
            ObjectInfo::TyreStack3Big(o) => o.heading = heading,
            ObjectInfo::TyreStack4Big(o) => o.heading = heading,
            ObjectInfo::MarkerCorner(o) => o.heading = heading,
            ObjectInfo::MarkerDistance(o) => o.heading = heading,
            ObjectInfo::LetterboardWY(o) => o.heading = heading,
            ObjectInfo::LetterboardRB(o) => o.heading = heading,
            ObjectInfo::Armco1(o) => o.heading = heading,
            ObjectInfo::Armco3(o) => o.heading = heading,
            ObjectInfo::Armco5(o) => o.heading = heading,
            ObjectInfo::BarrierLong(o) => o.heading = heading,
            ObjectInfo::BarrierRed(o) => o.heading = heading,
            ObjectInfo::BarrierWhite(o) => o.heading = heading,
            ObjectInfo::Banner(o) => o.heading = heading,
            ObjectInfo::Ramp1(o) => o.heading = heading,
            ObjectInfo::Ramp2(o) => o.heading = heading,
            ObjectInfo::VehicleSUV(o) => o.heading = heading,
            ObjectInfo::VehicleVan(o) => o.heading = heading,
            ObjectInfo::VehicleTruck(o) => o.heading = heading,
            ObjectInfo::VehicleAmbulance(o) => o.heading = heading,
            ObjectInfo::SpeedHump10M(o) => o.heading = heading,
            ObjectInfo::SpeedHump6M(o) => o.heading = heading,
            ObjectInfo::SpeedHump2M(o) => o.heading = heading,
            ObjectInfo::SpeedHump1M(o) => o.heading = heading,
            ObjectInfo::Kerb(o) => o.heading = heading,
            ObjectInfo::Post(o) => o.heading = heading,
            ObjectInfo::Marquee(o) => o.heading = heading,
            ObjectInfo::Bale(o) => o.heading = heading,
            ObjectInfo::Bin1(o) => o.heading = heading,
            ObjectInfo::Bin2(o) => o.heading = heading,
            ObjectInfo::Railing1(o) => o.heading = heading,
            ObjectInfo::Railing2(o) => o.heading = heading,
            ObjectInfo::StartLights1(o) => o.heading = heading,
            ObjectInfo::StartLights2(o) => o.heading = heading,
            ObjectInfo::StartLights3(o) => o.heading = heading,
            ObjectInfo::SignMetal(o) => o.heading = heading,
            ObjectInfo::SignSpeed(o) => o.heading = heading,
            ObjectInfo::ConcreteSlab(o) => o.heading = heading,
            ObjectInfo::ConcreteRamp(o) => o.heading = heading,
            ObjectInfo::ConcreteWall(o) => o.heading = heading,
            ObjectInfo::ConcretePillar(o) => o.heading = heading,
            ObjectInfo::ConcreteSlabWall(o) => o.heading = heading,
            ObjectInfo::ConcreteRampWall(o) => o.heading = heading,
            ObjectInfo::ConcreteShortSlabWall(o) => o.heading = heading,
            ObjectInfo::ConcreteWedge(o) => o.heading = heading,
            ObjectInfo::StartPosition(o) => o.heading = heading,
            ObjectInfo::PitStartPoint(o) => o.heading = heading,
            ObjectInfo::PitStopBox(o) => o.heading = heading,
            ObjectInfo::ChevronLeft(o) => o.heading = heading,
            ObjectInfo::ChevronRight(o) => o.heading = heading,
            ObjectInfo::InsimCircle(_)
            | ObjectInfo::RestrictedArea(_)
            | ObjectInfo::RouteChecker(_) => return false,
        }

        true
    }

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

    fn heading_objectinfo_wire(&self) -> u8 {
        match self {
            ObjectInfo::Control(i) => i.heading.to_objectinfo_wire(),
            ObjectInfo::Marshal(i) => i.heading.to_objectinfo_wire(),
            ObjectInfo::InsimCheckpoint(i) => i.heading.to_objectinfo_wire(),
            ObjectInfo::InsimCircle(i) => i.index,
            ObjectInfo::RestrictedArea(i) => i.radius,
            ObjectInfo::RouteChecker(i) => i.radius,
            ObjectInfo::ChalkLine(i) => i.heading.to_objectinfo_wire(),
            ObjectInfo::ChalkLine2(i) => i.heading.to_objectinfo_wire(),
            ObjectInfo::ChalkAhead(i) => i.heading.to_objectinfo_wire(),
            ObjectInfo::ChalkAhead2(i) => i.heading.to_objectinfo_wire(),
            ObjectInfo::ChalkLeft(i) => i.heading.to_objectinfo_wire(),
            ObjectInfo::ChalkLeft2(i) => i.heading.to_objectinfo_wire(),
            ObjectInfo::ChalkLeft3(i) => i.heading.to_objectinfo_wire(),
            ObjectInfo::ChalkRight(i) => i.heading.to_objectinfo_wire(),
            ObjectInfo::ChalkRight2(i) => i.heading.to_objectinfo_wire(),
            ObjectInfo::ChalkRight3(i) => i.heading.to_objectinfo_wire(),
            ObjectInfo::PaintLetters(i) => i.heading.to_objectinfo_wire(),
            ObjectInfo::PaintArrows(i) => i.heading.to_objectinfo_wire(),
            ObjectInfo::Cone1(i) => i.heading.to_objectinfo_wire(),
            ObjectInfo::Cone2(i) => i.heading.to_objectinfo_wire(),
            ObjectInfo::ConeTall1(i) => i.heading.to_objectinfo_wire(),
            ObjectInfo::ConeTall2(i) => i.heading.to_objectinfo_wire(),
            ObjectInfo::ConePointer(i) => i.heading.to_objectinfo_wire(),
            ObjectInfo::TyreSingle(i) => i.heading.to_objectinfo_wire(),
            ObjectInfo::TyreStack2(i) => i.heading.to_objectinfo_wire(),
            ObjectInfo::TyreStack3(i) => i.heading.to_objectinfo_wire(),
            ObjectInfo::TyreStack4(i) => i.heading.to_objectinfo_wire(),
            ObjectInfo::TyreSingleBig(i) => i.heading.to_objectinfo_wire(),
            ObjectInfo::TyreStack2Big(i) => i.heading.to_objectinfo_wire(),
            ObjectInfo::TyreStack3Big(i) => i.heading.to_objectinfo_wire(),
            ObjectInfo::TyreStack4Big(i) => i.heading.to_objectinfo_wire(),
            ObjectInfo::MarkerCorner(i) => i.heading.to_objectinfo_wire(),
            ObjectInfo::MarkerDistance(i) => i.heading.to_objectinfo_wire(),
            ObjectInfo::LetterboardWY(i) => i.heading.to_objectinfo_wire(),
            ObjectInfo::LetterboardRB(i) => i.heading.to_objectinfo_wire(),
            ObjectInfo::Armco1(i) => i.heading.to_objectinfo_wire(),
            ObjectInfo::Armco3(i) => i.heading.to_objectinfo_wire(),
            ObjectInfo::Armco5(i) => i.heading.to_objectinfo_wire(),
            ObjectInfo::BarrierLong(i) => i.heading.to_objectinfo_wire(),
            ObjectInfo::BarrierRed(i) => i.heading.to_objectinfo_wire(),
            ObjectInfo::BarrierWhite(i) => i.heading.to_objectinfo_wire(),
            ObjectInfo::Banner(i) => i.heading.to_objectinfo_wire(),
            ObjectInfo::Ramp1(i) => i.heading.to_objectinfo_wire(),
            ObjectInfo::Ramp2(i) => i.heading.to_objectinfo_wire(),
            ObjectInfo::VehicleSUV(i) => i.heading.to_objectinfo_wire(),
            ObjectInfo::VehicleVan(i) => i.heading.to_objectinfo_wire(),
            ObjectInfo::VehicleTruck(i) => i.heading.to_objectinfo_wire(),
            ObjectInfo::VehicleAmbulance(i) => i.heading.to_objectinfo_wire(),
            ObjectInfo::Kerb(i) => i.heading.to_objectinfo_wire(),
            ObjectInfo::Post(i) => i.heading.to_objectinfo_wire(),
            ObjectInfo::Marquee(i) => i.heading.to_objectinfo_wire(),
            ObjectInfo::Bale(i) => i.heading.to_objectinfo_wire(),
            ObjectInfo::SpeedHump10M(i) => i.heading.to_objectinfo_wire(),
            ObjectInfo::SpeedHump6M(i) => i.heading.to_objectinfo_wire(),
            ObjectInfo::SpeedHump2M(i) => i.heading.to_objectinfo_wire(),
            ObjectInfo::SpeedHump1M(i) => i.heading.to_objectinfo_wire(),
            ObjectInfo::Bin1(i) => i.heading.to_objectinfo_wire(),
            ObjectInfo::Bin2(i) => i.heading.to_objectinfo_wire(),
            ObjectInfo::Railing1(i) => i.heading.to_objectinfo_wire(),
            ObjectInfo::Railing2(i) => i.heading.to_objectinfo_wire(),
            ObjectInfo::StartLights1(i) => i.heading.to_objectinfo_wire(),
            ObjectInfo::StartLights2(i) => i.heading.to_objectinfo_wire(),
            ObjectInfo::StartLights3(i) => i.heading.to_objectinfo_wire(),
            ObjectInfo::SignMetal(i) => i.heading.to_objectinfo_wire(),
            ObjectInfo::ChevronLeft(i) => i.heading.to_objectinfo_wire(),
            ObjectInfo::ChevronRight(i) => i.heading.to_objectinfo_wire(),
            ObjectInfo::SignSpeed(i) => i.heading.to_objectinfo_wire(),
            ObjectInfo::ConcreteSlab(i) => i.heading.to_objectinfo_wire(),
            ObjectInfo::ConcreteRamp(i) => i.heading.to_objectinfo_wire(),
            ObjectInfo::ConcreteWall(i) => i.heading.to_objectinfo_wire(),
            ObjectInfo::ConcretePillar(i) => i.heading.to_objectinfo_wire(),
            ObjectInfo::ConcreteSlabWall(i) => i.heading.to_objectinfo_wire(),
            ObjectInfo::ConcreteRampWall(i) => i.heading.to_objectinfo_wire(),
            ObjectInfo::ConcreteShortSlabWall(i) => i.heading.to_objectinfo_wire(),
            ObjectInfo::ConcreteWedge(i) => i.heading.to_objectinfo_wire(),
            ObjectInfo::StartPosition(i) => i.heading.to_objectinfo_wire(),
            ObjectInfo::PitStartPoint(i) => i.heading.to_objectinfo_wire(),
            ObjectInfo::PitStopBox(i) => i.heading.to_objectinfo_wire(),
        }
    }
}

macro_rules! generate_object_info_accessors {
    ($($variant:ident),* $(,)?) => {
        impl ObjectInfo {
            /// XYZ object position in raw scale
            pub fn position(&self) -> &ObjectCoordinate {
                match self {
                    $(
                        ObjectInfo::$variant(i) => &i.xyz,
                    )*
                }
            }

            /// Mutable XYZ position in raw scale
            pub fn position_mut(&mut self) -> &mut ObjectCoordinate {
                match self {
                    $(
                        ObjectInfo::$variant(i) => &mut i.xyz,
                    )*
                }
            }

            fn flags(&self) -> ObjectFlags {
                match self {
                    $(
                        ObjectInfo::$variant(i) => i.to_flags(),
                    )*
                }
            }
        }
    };
}

generate_object_info_accessors!(
    Control,
    Marshal,
    InsimCheckpoint,
    InsimCircle,
    RestrictedArea,
    RouteChecker,
    ChalkLine,
    ChalkLine2,
    ChalkAhead,
    ChalkAhead2,
    ChalkLeft,
    ChalkLeft2,
    ChalkLeft3,
    ChalkRight,
    ChalkRight2,
    ChalkRight3,
    PaintLetters,
    PaintArrows,
    Cone1,
    Cone2,
    ConeTall1,
    ConeTall2,
    ConePointer,
    TyreSingle,
    TyreStack2,
    TyreStack3,
    TyreStack4,
    TyreSingleBig,
    TyreStack2Big,
    TyreStack3Big,
    TyreStack4Big,
    MarkerCorner,
    MarkerDistance,
    LetterboardWY,
    LetterboardRB,
    Armco1,
    Armco3,
    Armco5,
    BarrierLong,
    BarrierRed,
    BarrierWhite,
    Banner,
    Ramp1,
    Ramp2,
    VehicleSUV,
    VehicleVan,
    VehicleTruck,
    VehicleAmbulance,
    Kerb,
    Post,
    Marquee,
    Bale,
    SpeedHump10M,
    SpeedHump6M,
    SpeedHump2M,
    SpeedHump1M,
    Bin1,
    Bin2,
    Railing1,
    Railing2,
    StartLights1,
    StartLights2,
    StartLights3,
    SignMetal,
    ChevronLeft,
    ChevronRight,
    SignSpeed,
    ConcreteSlab,
    ConcreteRamp,
    ConcreteWall,
    ConcretePillar,
    ConcreteSlabWall,
    ConcreteRampWall,
    ConcreteShortSlabWall,
    ConcreteWedge,
    StartPosition,
    PitStartPoint,
    PitStopBox
);

impl Encode for ObjectInfo {
    fn encode(&self, buf: &mut bytes::BytesMut) -> Result<(), EncodeError> {
        let index: u8 = self.into();
        let xyz = self.position();
        let flags = self.flags();
        let heading = self.heading_objectinfo_wire();

        xyz.x.encode(buf)?;
        xyz.y.encode(buf)?;
        xyz.z.encode(buf)?;
        flags.0.encode(buf)?;
        index.encode(buf)?;
        heading.encode(buf)?;

        Ok(())
    }
}
