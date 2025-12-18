//! Objects are used in both insim and lyt files

pub mod armco1;
pub mod armco3;
pub mod armco5;
pub mod bale;
pub mod banner;
pub mod barrier_long;
pub mod barrier_red;
pub mod barrier_white;
pub mod bin1;
pub mod bin2;
pub mod chalk_ahead;
pub mod chalk_ahead2;
pub mod chalk_left;
pub mod chalk_left2;
pub mod chalk_left3;
pub mod chalk_line;
pub mod chalk_line2;
pub mod chalk_right;
pub mod chalk_right2;
pub mod chalk_right3;
pub mod concrete;
pub mod cone1;
pub mod cone2;
pub mod cone_pointer;
pub mod cone_tall1;
pub mod cone_tall2;
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
pub mod railing1;
pub mod railing2;
pub mod ramp1;
pub mod ramp2;
pub mod sign_metal;
pub mod sign_speed;
pub mod speed_hump_10m;
pub mod speed_hump_1m;
pub mod speed_hump_2m;
pub mod speed_hump_6m;
pub mod start_lights1;
pub mod start_lights2;
pub mod start_lights3;
pub mod start_position;
pub mod tyre_single;
pub mod tyre_single_big;
pub mod tyre_stack2;
pub mod tyre_stack2_big;
pub mod tyre_stack3;
pub mod tyre_stack3_big;
pub mod tyre_stack4;
pub mod tyre_stack4_big;
pub mod vehicle_ambulance;
pub mod vehicle_suv;
pub mod vehicle_truck;
pub mod vehicle_van;

use crate::{Decode, DecodeError, Encode, EncodeError};

/// Wire format representation for object encoding/decoding
#[derive(Debug, Clone, Copy)]
pub(crate) struct ObjectWire {
    /// Object index/type discriminator
    pub index: u8,
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
    /// Encode this Object to wire format
    fn to_wire(&self) -> Result<ObjectWire, EncodeError>;
    /// Decode Object from wire format
    fn from_wire(wire: ObjectWire) -> Result<Self, DecodeError>;
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Layout Object
pub struct ObjectInfo {
    /// Object xyz position
    pub xyz: glam::I16Vec3,
    /// Kind
    pub kind: ObjectKind,
}

#[derive(Debug, Clone, from_variants::FromVariants)]
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
    ChalkLine(chalk_line::ChalkLine),
    /// ChalkLine2
    ChalkLine2(chalk_line2::ChalkLine2),
    /// ChalkAhead
    ChalkAhead(chalk_ahead::ChalkAhead),
    /// ChalkAhead2
    ChalkAhead2(chalk_ahead2::ChalkAhead2),
    /// ChalkLeft
    ChalkLeft(chalk_left::ChalkLeft),
    /// ChalkLeft2
    ChalkLeft2(chalk_left2::ChalkLeft2),
    /// ChalkLeft3
    ChalkLeft3(chalk_left3::ChalkLeft3),
    /// ChalkRight
    ChalkRight(chalk_right::ChalkRight),
    /// ChalkRight2
    ChalkRight2(chalk_right2::ChalkRight2),
    /// ChalkRight3
    ChalkRight3(chalk_right3::ChalkRight3),
    /// Painted Letters
    PaintLetters(painted::Letters),
    /// Painted Arrows
    PaintArrows(painted::Arrows),
    /// Cone1
    Cone1(cone1::Cone1),
    /// Cone2
    Cone2(cone2::Cone2),
    /// ConeTall1
    ConeTall1(cone_tall1::ConeTall1),
    /// ConeTall2
    ConeTall2(cone_tall2::ConeTall2),
    /// Cone Pointer
    ConePointer(cone_pointer::ConePointer),
    /// Tyre Single
    TyreSingle(tyre_single::TyreSingle),
    /// Tyre Stack2
    TyreStack2(tyre_stack2::TyreStack2),
    /// Tyre Stack3
    TyreStack3(tyre_stack3::TyreStack3),
    /// Tyre Stack4
    TyreStack4(tyre_stack4::TyreStack4),
    /// Tyre Single Big
    TyreSingleBig(tyre_single_big::TyreSingleBig),
    /// Tyre Stack2 Big
    TyreStack2Big(tyre_stack2_big::TyreStack2Big),
    /// Tyre Stack3 Big
    TyreStack3Big(tyre_stack3_big::TyreStack3Big),
    /// Tyre Stack4 Big
    TyreStack4Big(tyre_stack4_big::TyreStack4Big),
    /// Corner Marker
    MarkerCorner(marker::MarkerCorner),
    /// Distance Marker
    MarkerDistance(marker::MarkerDistance),
    /// Letterboard WY
    LetterboardWY(letterboard_wy::LetterboardWY),
    /// Letterboard RB
    LetterboardRB(letterboard_rb::LetterboardRB),
    /// Armco1
    Armco1(armco1::Armco1),
    /// Armco3
    Armco3(armco3::Armco3),
    /// Armco5
    Armco5(armco5::Armco5),
    /// Barrier Long
    BarrierLong(barrier_long::BarrierLong),
    /// Barrier Red
    BarrierRed(barrier_red::BarrierRed),
    /// Barrier White
    BarrierWhite(barrier_white::BarrierWhite),
    /// Banner
    Banner(banner::Banner),
    /// Ramp1
    Ramp1(ramp1::Ramp1),
    /// Ramp2
    Ramp2(ramp2::Ramp2),
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
    SpeedHump10M(speed_hump_10m::SpeedHump10M),
    /// Speed hump 6m
    SpeedHump6M(speed_hump_6m::SpeedHump6M),
    /// Speed hump 2m
    SpeedHump2M(speed_hump_2m::SpeedHump2M),
    /// Speed hump 1m
    SpeedHump1M(speed_hump_1m::SpeedHump1M),
    /// Bin1
    Bin1(bin1::Bin1),
    /// Bin2
    Bin2(bin2::Bin2),
    /// Railing1
    Railing1(railing1::Railing1),
    /// Railing2
    Railing2(railing2::Railing2),
    /// Start lights 1
    StartLights1(start_lights1::StartLights1),
    /// Start lights 2
    StartLights2(start_lights2::StartLights2),
    /// Start lights 3
    StartLights3(start_lights3::StartLights3),
    /// Metal Sign
    SignMetal(sign_metal::SignMetal),
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

impl Decode for ObjectInfo {
    fn decode(buf: &mut bytes::Bytes) -> Result<Self, DecodeError> {
        let x = i16::decode(buf)?;
        let y = i16::decode(buf)?;
        let z = u8::decode(buf)?;

        let flags = u8::decode(buf)?;
        let index = u8::decode(buf)?;
        let heading = u8::decode(buf)?;

        let wire = ObjectWire {
            index,
            flags,
            heading,
        };

        let kind = match index {
            0 => ObjectKind::Control(control::Control::decode(wire)?),
            240 => ObjectKind::Marshal(marshal::Marshal::decode(wire)?),
            252 => ObjectKind::InsimCheckpoint(insim::InsimCheckpoint::decode(wire)?),
            253 => ObjectKind::InsimCircle(insim::InsimCircle::decode(wire)?),
            254 => ObjectKind::RestrictedArea(marshal::RestrictedArea::decode(wire)?),
            255 => ObjectKind::RouteChecker(marshal::RouteChecker::decode(wire)?),

            4 => ObjectKind::ChalkLine(chalk_line::ChalkLine::from_wire(wire)?),
            5 => ObjectKind::ChalkLine2(chalk_line2::ChalkLine2::from_wire(wire)?),
            6 => ObjectKind::ChalkAhead(chalk_ahead::ChalkAhead::from_wire(wire)?),
            7 => ObjectKind::ChalkAhead2(chalk_ahead2::ChalkAhead2::from_wire(wire)?),
            8 => ObjectKind::ChalkLeft(chalk_left::ChalkLeft::from_wire(wire)?),
            9 => ObjectKind::ChalkLeft2(chalk_left2::ChalkLeft2::from_wire(wire)?),
            10 => ObjectKind::ChalkLeft3(chalk_left3::ChalkLeft3::from_wire(wire)?),
            11 => ObjectKind::ChalkRight(chalk_right::ChalkRight::from_wire(wire)?),
            12 => ObjectKind::ChalkRight2(chalk_right2::ChalkRight2::from_wire(wire)?),
            13 => ObjectKind::ChalkRight3(chalk_right3::ChalkRight3::from_wire(wire)?),
            16 => ObjectKind::PaintLetters(painted::Letters::from_wire(wire)?),
            17 => ObjectKind::PaintArrows(painted::Arrows::from_wire(wire)?),
            20 => ObjectKind::Cone1(cone1::Cone1::from_wire(wire)?),
            21 => ObjectKind::Cone2(cone2::Cone2::from_wire(wire)?),
            32 => ObjectKind::ConeTall1(cone_tall1::ConeTall1::from_wire(wire)?),
            33 => ObjectKind::ConeTall2(cone_tall2::ConeTall2::from_wire(wire)?),
            40 => ObjectKind::ConePointer(cone_pointer::ConePointer::from_wire(wire)?),

            48 => ObjectKind::TyreSingle(tyre_single::TyreSingle::from_wire(wire)?),
            49 => ObjectKind::TyreStack2(tyre_stack2::TyreStack2::from_wire(wire)?),
            50 => ObjectKind::TyreStack3(tyre_stack3::TyreStack3::from_wire(wire)?),
            51 => ObjectKind::TyreStack4(tyre_stack4::TyreStack4::from_wire(wire)?),
            52 => ObjectKind::TyreSingleBig(tyre_single_big::TyreSingleBig::from_wire(wire)?),
            53 => ObjectKind::TyreStack2Big(tyre_stack2_big::TyreStack2Big::from_wire(wire)?),
            54 => ObjectKind::TyreStack3Big(tyre_stack3_big::TyreStack3Big::from_wire(wire)?),
            55 => ObjectKind::TyreStack4Big(tyre_stack4_big::TyreStack4Big::from_wire(wire)?),

            62 => ObjectKind::MarkerCorner(marker::MarkerCorner::from_wire(wire)?),
            84 => ObjectKind::MarkerDistance(marker::MarkerDistance::from_wire(wire)?),
            92 => ObjectKind::LetterboardWY(letterboard_wy::LetterboardWY::from_wire(wire)?),
            93 => ObjectKind::LetterboardRB(letterboard_rb::LetterboardRB::from_wire(wire)?),
            96 => ObjectKind::Armco1(armco1::Armco1::from_wire(wire)?),
            97 => ObjectKind::Armco3(armco3::Armco3::from_wire(wire)?),
            98 => ObjectKind::Armco5(armco5::Armco5::from_wire(wire)?),
            104 => ObjectKind::BarrierLong(barrier_long::BarrierLong::from_wire(wire)?),
            105 => ObjectKind::BarrierRed(barrier_red::BarrierRed::from_wire(wire)?),
            106 => ObjectKind::BarrierWhite(barrier_white::BarrierWhite::from_wire(wire)?),
            112 => ObjectKind::Banner(banner::Banner::from_wire(wire)?),
            120 => ObjectKind::Ramp1(ramp1::Ramp1::from_wire(wire)?),
            121 => ObjectKind::Ramp2(ramp2::Ramp2::from_wire(wire)?),
            124 => ObjectKind::VehicleSUV(vehicle_suv::VehicleSUV::from_wire(wire)?),
            125 => ObjectKind::VehicleVan(vehicle_van::VehicleVan::from_wire(wire)?),
            126 => ObjectKind::VehicleTruck(vehicle_truck::VehicleTruck::from_wire(wire)?),
            127 => {
                ObjectKind::VehicleAmbulance(vehicle_ambulance::VehicleAmbulance::from_wire(wire)?)
            },
            128 => ObjectKind::SpeedHump10M(speed_hump_10m::SpeedHump10M::from_wire(wire)?),
            129 => ObjectKind::SpeedHump6M(speed_hump_6m::SpeedHump6M::from_wire(wire)?),
            130 => ObjectKind::SpeedHump2M(speed_hump_2m::SpeedHump2M::from_wire(wire)?),
            131 => ObjectKind::SpeedHump1M(speed_hump_1m::SpeedHump1M::from_wire(wire)?),
            132 => ObjectKind::Kerb(kerb::Kerb::from_wire(wire)?),
            136 => ObjectKind::Post(post::Post::from_wire(wire)?),
            140 => ObjectKind::Marquee(marquee::Marquee::from_wire(wire)?),
            144 => ObjectKind::Bale(bale::Bale::from_wire(wire)?),
            145 => ObjectKind::Bin1(bin1::Bin1::from_wire(wire)?),
            146 => ObjectKind::Bin2(bin2::Bin2::from_wire(wire)?),
            147 => ObjectKind::Railing1(railing1::Railing1::from_wire(wire)?),
            148 => ObjectKind::Railing2(railing2::Railing2::from_wire(wire)?),
            149 => ObjectKind::StartLights1(start_lights1::StartLights1::from_wire(wire)?),
            150 => ObjectKind::StartLights2(start_lights2::StartLights2::from_wire(wire)?),
            151 => ObjectKind::StartLights3(start_lights3::StartLights3::from_wire(wire)?),
            160 => ObjectKind::SignMetal(sign_metal::SignMetal::from_wire(wire)?),
            168 => ObjectKind::SignSpeed(sign_speed::SignSpeed::from_wire(wire)?),
            172 => ObjectKind::ConcreteSlab(concrete::ConcreteSlab::from_wire(wire)?),
            173 => ObjectKind::ConcreteRamp(concrete::ConcreteRamp::from_wire(wire)?),
            174 => ObjectKind::ConcreteWall(concrete::ConcreteWall::from_wire(wire)?),
            175 => ObjectKind::ConcretePillar(concrete::ConcretePillar::from_wire(wire)?),
            176 => ObjectKind::ConcreteSlabWall(concrete::ConcreteSlabWall::from_wire(wire)?),
            177 => ObjectKind::ConcreteRampWall(concrete::ConcreteRampWall::from_wire(wire)?),
            178 => {
                ObjectKind::ConcreteShortSlabWall(concrete::ConcreteShortSlabWall::from_wire(wire)?)
            },
            179 => ObjectKind::ConcreteWedge(concrete::ConcreteWedge::from_wire(wire)?),
            184 => ObjectKind::StartPosition(start_position::StartPosition::from_wire(wire)?),
            185 => ObjectKind::PitStartPoint(pit_start_point::PitStartPoint::from_wire(wire)?),
            186 => ObjectKind::PitStopBox(pit::PitStopBox::from_wire(wire)?),

            _ => {
                return Err(DecodeError::NoVariantMatch {
                    found: index as u64,
                });
            },
        };

        Ok(Self {
            xyz: glam::I16Vec3 { x, y, z: z as i16 },
            kind,
        })
    }
}

impl ObjectKind {
    /// Get heading if this object has one
    pub fn heading(&self) -> Option<crate::direction::Direction> {
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
        }
    }
}

impl ObjectInfo {
    /// Create spaced-out objects starting from the left (extends to the right)
    pub fn spaced_from_left(
        objects: impl IntoIterator<Item = ObjectKind>,
        start_pos: glam::I16Vec3,
        spacing: i16,
    ) -> Vec<ObjectInfo> {
        let mut result = Vec::new();
        let mut x = start_pos.x;

        for kind in objects {
            result.push(ObjectInfo {
                xyz: glam::I16Vec3 {
                    x,
                    y: start_pos.y,
                    z: start_pos.z,
                },
                kind,
            });
            x += spacing;
        }
        result
    }

    /// Create spaced-out objects ending at the right (extends to the left)
    pub fn spaced_from_right(
        objects: impl IntoIterator<Item = ObjectKind>,
        end_pos: glam::I16Vec3,
        spacing: i16,
    ) -> Vec<ObjectInfo> {
        let objects_vec: Vec<_> = objects.into_iter().collect();
        let total_width = (objects_vec.len() as i16 - 1) * spacing;
        let mut result = Vec::new();
        let mut x = end_pos.x - total_width;

        for kind in objects_vec {
            result.push(ObjectInfo {
                xyz: glam::I16Vec3 {
                    x,
                    y: end_pos.y,
                    z: end_pos.z,
                },
                kind,
            });
            x += spacing;
        }
        result
    }

    /// Create spaced-out objects centered at the given position
    pub fn spaced_from_center(
        objects: impl IntoIterator<Item = ObjectKind>,
        center_pos: glam::I16Vec3,
        spacing: i16,
    ) -> Vec<ObjectInfo> {
        let objects_vec: Vec<_> = objects.into_iter().collect();
        let count = objects_vec.len() as i16;
        let total_width = (count - 1) * spacing;
        let start_x = center_pos.x - total_width / 2;
        let mut result = Vec::new();
        let mut x = start_x;

        for kind in objects_vec {
            result.push(ObjectInfo {
                xyz: glam::I16Vec3 {
                    x,
                    y: center_pos.y,
                    z: center_pos.z,
                },
                kind,
            });
            x += spacing;
        }
        result
    }
}

impl Encode for ObjectInfo {
    fn encode(&self, buf: &mut bytes::BytesMut) -> Result<(), EncodeError> {
        self.xyz.x.encode(buf)?;
        self.xyz.y.encode(buf)?;
        (self.xyz.z as u8).encode(buf)?; // FIXME: use TryFrom
        let wire = match &self.kind {
            ObjectKind::Control(control) => control.encode()?,
            ObjectKind::Marshal(marshal) => marshal.encode()?,
            ObjectKind::InsimCheckpoint(insim_checkpoint) => insim_checkpoint.encode()?,
            ObjectKind::InsimCircle(insim_circle) => insim_circle.encode()?,
            ObjectKind::RestrictedArea(restricted_area) => restricted_area.encode()?,
            ObjectKind::RouteChecker(route_checker) => route_checker.encode()?,
            ObjectKind::ChalkLine(chalk) => chalk.to_wire()?,
            ObjectKind::ChalkLine2(chalk) => chalk.to_wire()?,
            ObjectKind::ChalkAhead(chalk) => chalk.to_wire()?,
            ObjectKind::ChalkAhead2(chalk) => chalk.to_wire()?,
            ObjectKind::ChalkLeft(chalk) => chalk.to_wire()?,
            ObjectKind::ChalkLeft2(chalk) => chalk.to_wire()?,
            ObjectKind::ChalkLeft3(chalk) => chalk.to_wire()?,
            ObjectKind::ChalkRight(chalk) => chalk.to_wire()?,
            ObjectKind::ChalkRight2(chalk) => chalk.to_wire()?,
            ObjectKind::ChalkRight3(chalk) => chalk.to_wire()?,
            ObjectKind::PaintLetters(letters) => letters.to_wire()?,
            ObjectKind::PaintArrows(arrows) => arrows.to_wire()?,
            ObjectKind::Cone1(cone1) => cone1.to_wire()?,
            ObjectKind::Cone2(cone2) => cone2.to_wire()?,
            ObjectKind::ConeTall1(cone_tall1) => cone_tall1.to_wire()?,
            ObjectKind::ConeTall2(cone_tall2) => cone_tall2.to_wire()?,
            ObjectKind::ConePointer(cone_pointer) => cone_pointer.to_wire()?,
            ObjectKind::TyreSingle(tyre) => tyre.to_wire()?,
            ObjectKind::TyreStack2(tyre) => tyre.to_wire()?,
            ObjectKind::TyreStack3(tyre) => tyre.to_wire()?,
            ObjectKind::TyreStack4(tyre) => tyre.to_wire()?,
            ObjectKind::TyreSingleBig(tyre) => tyre.to_wire()?,
            ObjectKind::TyreStack2Big(tyre) => tyre.to_wire()?,
            ObjectKind::TyreStack3Big(tyre) => tyre.to_wire()?,
            ObjectKind::TyreStack4Big(tyre) => tyre.to_wire()?,
            ObjectKind::MarkerCorner(marker_corner) => marker_corner.to_wire()?,
            ObjectKind::MarkerDistance(marker_distance) => marker_distance.to_wire()?,
            ObjectKind::LetterboardWY(letterboard_wy) => letterboard_wy.to_wire()?,
            ObjectKind::LetterboardRB(letterboard_rb) => letterboard_rb.to_wire()?,
            ObjectKind::Armco1(armco1) => armco1.to_wire()?,
            ObjectKind::Armco3(armco3) => armco3.to_wire()?,
            ObjectKind::Armco5(armco5) => armco5.to_wire()?,
            ObjectKind::BarrierLong(barrier) => barrier.to_wire()?,
            ObjectKind::BarrierRed(barrier) => barrier.to_wire()?,
            ObjectKind::BarrierWhite(barrier) => barrier.to_wire()?,
            ObjectKind::Banner(banner) => banner.to_wire()?,
            ObjectKind::Ramp1(ramp1) => ramp1.to_wire()?,
            ObjectKind::Ramp2(ramp2) => ramp2.to_wire()?,
            ObjectKind::VehicleSUV(veh) => veh.to_wire()?,
            ObjectKind::VehicleVan(veh) => veh.to_wire()?,
            ObjectKind::VehicleTruck(veh) => veh.to_wire()?,
            ObjectKind::VehicleAmbulance(veh) => veh.to_wire()?,
            ObjectKind::SpeedHump10M(speed_hump) => speed_hump.to_wire()?,
            ObjectKind::SpeedHump6M(speed_hump) => speed_hump.to_wire()?,
            ObjectKind::SpeedHump2M(speed_hump) => speed_hump.to_wire()?,
            ObjectKind::SpeedHump1M(speed_hump) => speed_hump.to_wire()?,
            ObjectKind::Kerb(kerb) => kerb.to_wire()?,
            ObjectKind::Post(post) => post.to_wire()?,
            ObjectKind::Marquee(marquee) => marquee.to_wire()?,
            ObjectKind::Bale(bale) => bale.to_wire()?,
            ObjectKind::Bin1(bin1) => bin1.to_wire()?,
            ObjectKind::Bin2(bin2) => bin2.to_wire()?,
            ObjectKind::Railing1(railing1) => railing1.to_wire()?,
            ObjectKind::Railing2(railing2) => railing2.to_wire()?,
            ObjectKind::StartLights1(start_lights1) => start_lights1.to_wire()?,
            ObjectKind::StartLights2(start_lights2) => start_lights2.to_wire()?,
            ObjectKind::StartLights3(start_lights3) => start_lights3.to_wire()?,
            ObjectKind::SignMetal(sign_metal) => sign_metal.to_wire()?,
            ObjectKind::SignSpeed(sign_speed) => sign_speed.to_wire()?,
            ObjectKind::ConcreteSlab(concrete_slab) => concrete_slab.to_wire()?,
            ObjectKind::ConcreteRamp(concrete_ramp) => concrete_ramp.to_wire()?,
            ObjectKind::ConcreteWall(concrete_wall) => concrete_wall.to_wire()?,
            ObjectKind::ConcretePillar(concrete_pillar) => concrete_pillar.to_wire()?,
            ObjectKind::ConcreteSlabWall(concrete_slab_wall) => concrete_slab_wall.to_wire()?,
            ObjectKind::ConcreteRampWall(concrete_ramp_wall) => concrete_ramp_wall.to_wire()?,
            ObjectKind::ConcreteShortSlabWall(concrete_short_slab_wall) => {
                concrete_short_slab_wall.to_wire()?
            },
            ObjectKind::ConcreteWedge(concrete_wedge) => concrete_wedge.to_wire()?,
            ObjectKind::StartPosition(start_position) => start_position.to_wire()?,
            ObjectKind::PitStartPoint(pit_start_point) => pit_start_point.to_wire()?,
            ObjectKind::PitStopBox(pit_stop_box) => pit_stop_box.to_wire()?,
        };
        wire.flags.encode(buf)?;
        wire.index.encode(buf)?;
        wire.heading.encode(buf)?;

        Ok(())
    }
}
