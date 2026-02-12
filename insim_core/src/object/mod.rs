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
#[cfg(test)]
mod tests;

pub use object_coordinate::ObjectCoordinate;

use crate::{Decode, DecodeError, Encode, EncodeError, heading::Heading};

trait ObjectInfoInner {
    fn flags(&self) -> u8;

    fn heading_mut(&mut self) -> Option<&mut Heading> {
        None
    }

    fn heading(&self) -> Option<Heading> {
        None
    }

    fn floating(&self) -> Option<bool> {
        None
    }

    fn heading_objectinfo_wire(&self) -> u8 {
        self.heading()
            .map(|h| h.to_objectinfo_wire())
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Raw layout object wire fields.
pub struct Raw {
    /// Index
    pub index: u8,
    /// Position
    pub xyz: ObjectCoordinate,
    /// Flags
    pub flags: u8,
    /// Raw heading
    pub heading: u8,
}

impl Raw {
    /// Check if the floating flag is set
    pub fn raw_floating(&self) -> bool {
        self.flags & 0x80 != 0
    }

    /// Helper to extract colour from flags (bits 0-2)
    pub fn raw_colour(&self) -> u8 {
        self.flags & 0x07
    }

    /// Helper to extract mapping from flags (bits 3-6)
    pub fn raw_mapping(&self) -> u8 {
        (self.flags >> 3) & 0x0f
    }
}

impl ObjectInfoInner for Raw {
    fn floating(&self) -> Option<bool> {
        Some(self.raw_floating())
    }

    fn flags(&self) -> u8 {
        self.flags
    }

    fn heading_objectinfo_wire(&self) -> u8 {
        self.heading
    }
}

macro_rules! define_object_info {
    (
        $(
            $(#[$variant_meta:meta])*
            $index:literal => $variant:ident($ty:path),
        )+
    ) => {
        #[derive(Debug, Clone)]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        #[non_exhaustive]
        /// Layout Object
        pub enum ObjectInfo {
            $(
                $(#[$variant_meta])*
                $variant($ty),
            )+
            /// Fallback
            Unknown(Raw),
        }

        impl Decode for ObjectInfo {
            fn decode(buf: &mut bytes::Bytes) -> Result<Self, DecodeError> {
                let x = i16::decode(buf).map_err(|e| e.nested().context("Raw::x"))?;
                let y = i16::decode(buf).map_err(|e| e.nested().context("Raw::y"))?;
                let z = u8::decode(buf).map_err(|e| e.nested().context("Raw::z"))?;
                let flags = u8::decode(buf).map_err(|e| e.nested().context("Raw::flags"))?;
                let index = u8::decode(buf).map_err(|e| e.nested().context("Raw::index"))?;
                let heading = u8::decode(buf).map_err(|e| e.nested().context("Raw::heading"))?;

                let raw = Raw {
                    index,
                    xyz: ObjectCoordinate { x, y, z },
                    flags,
                    heading,
                };

                match raw.index {
                    $(
                        $index => Ok(ObjectInfo::$variant(<$ty>::new(raw).map_err(|e| {
                            e.nested()
                                .context(concat!("ObjectInfo::", stringify!($variant)))
                        })?)),
                    )+
                    _ => Ok(ObjectInfo::Unknown(raw)),
                }
            }
        }

        impl Encode for ObjectInfo {
            fn encode(&self, buf: &mut bytes::BytesMut) -> Result<(), EncodeError> {
                let index: u8 = match self {
                    $(
                        ObjectInfo::$variant(_) => $index,
                    )+
                    ObjectInfo::Unknown(Raw { index, .. }) => *index,
                };

                let xyz = self.position();
                let flags = self.flags();
                let heading = self.heading_objectinfo_wire();

                xyz.x.encode(buf)?;
                xyz.y.encode(buf)?;
                xyz.z.encode(buf)?;
                flags.encode(buf)?;
                index.encode(buf)?;
                heading.encode(buf)?;

                Ok(())
            }
        }

        impl ObjectInfo {
            /// XYZ object position in raw scale
            pub fn position(&self) -> &ObjectCoordinate {
                match self {
                    $(
                        ObjectInfo::$variant(i) => &i.xyz,
                    )+
                    ObjectInfo::Unknown(i) => &i.xyz,
                }
            }

            /// Mutable XYZ position in raw scale
            pub fn position_mut(&mut self) -> &mut ObjectCoordinate {
                match self {
                    $(
                        ObjectInfo::$variant(i) => &mut i.xyz,
                    )+
                    ObjectInfo::Unknown(i) => &mut i.xyz,
                }
            }

            fn flags(&self) -> u8 {
                match self {
                    $(
                        ObjectInfo::$variant(i) => i.flags(),
                    )+
                    ObjectInfo::Unknown(i) => i.flags(),
                }
            }

            /// Get mutable heading if this object has one
            pub fn heading_mut(&mut self) -> Option<&mut Heading> {
                match self {
                    $(
                        ObjectInfo::$variant(i) => i.heading_mut(),
                    )+
                    ObjectInfo::Unknown(i) => i.heading_mut(),
                }
            }

            /// Get heading if this object has one
            pub fn heading(&self) -> Option<Heading> {
                match self {
                    $(
                        ObjectInfo::$variant(i) => i.heading(),
                    )+
                    ObjectInfo::Unknown(i) => i.heading(),
                }
            }

            /// Get floating flag if this object has one
            pub fn floating(&self) -> Option<bool> {
                match self {
                    $(
                        ObjectInfo::$variant(i) => i.floating(),
                    )+
                    ObjectInfo::Unknown(i) => i.floating(),
                }
            }

            fn heading_objectinfo_wire(&self) -> u8 {
                match self {
                    $(
                        ObjectInfo::$variant(i) => i.heading_objectinfo_wire(),
                    )+
                    ObjectInfo::Unknown(i) => i.heading_objectinfo_wire(),
                }
            }
        }
    };
}

define_object_info! {
    /// Control - start, finish, checkpoints
    0 => Control(control::Control),
    /// Marshal
    240 => Marshal(marshal::Marshal),
    /// Insim Checkpoint
    252 => InsimCheckpoint(insim::InsimCheckpoint),
    /// Insim circle
    253 => InsimCircle(insim::InsimCircle),
    /// Restrited area / circle
    254 => RestrictedArea(marshal::RestrictedArea),
    /// Route checker
    255 => RouteChecker(marshal::RouteChecker),
    /// ChalkLine
    4 => ChalkLine(chalk::Chalk),
    /// ChalkLine2
    5 => ChalkLine2(chalk::Chalk),
    /// ChalkAhead
    6 => ChalkAhead(chalk::Chalk),
    /// ChalkAhead2
    7 => ChalkAhead2(chalk::Chalk),
    /// ChalkLeft
    8 => ChalkLeft(chalk::Chalk),
    /// ChalkLeft2
    9 => ChalkLeft2(chalk::Chalk),
    /// ChalkLeft3
    10 => ChalkLeft3(chalk::Chalk),
    /// ChalkRight
    11 => ChalkRight(chalk::Chalk),
    /// ChalkRight2
    12 => ChalkRight2(chalk::Chalk),
    /// ChalkRight3
    13 => ChalkRight3(chalk::Chalk),
    /// Painted Letters
    16 => PaintLetters(painted::Letters),
    /// Painted Arrows
    17 => PaintArrows(painted::Arrows),
    /// Cone1
    20 => Cone1(cones::Cone),
    /// Cone2
    21 => Cone2(cones::Cone),
    /// ConeTall1
    32 => ConeTall1(cones::Cone),
    /// ConeTall2
    33 => ConeTall2(cones::Cone),
    /// Cone Pointer
    40 => ConePointer(cones::Cone),
    /// Tyre Single
    48 => TyreSingle(tyres::Tyres),
    /// Tyre Stack2
    49 => TyreStack2(tyres::Tyres),
    /// Tyre Stack3
    50 => TyreStack3(tyres::Tyres),
    /// Tyre Stack4
    51 => TyreStack4(tyres::Tyres),
    /// Tyre Single Big
    52 => TyreSingleBig(tyres::Tyres),
    /// Tyre Stack2 Big
    53 => TyreStack2Big(tyres::Tyres),
    /// Tyre Stack3 Big
    54 => TyreStack3Big(tyres::Tyres),
    /// Tyre Stack4 Big
    55 => TyreStack4Big(tyres::Tyres),
    /// Corner Marker
    64 => MarkerCorner(marker::MarkerCorner),
    /// Distance Marker
    84 => MarkerDistance(marker::MarkerDistance),
    /// Letterboard WY
    92 => LetterboardWY(letterboard_wy::LetterboardWY),
    /// Letterboard RB
    93 => LetterboardRB(letterboard_rb::LetterboardRB),
    /// Armco1
    96 => Armco1(armco::Armco),
    /// Armco3
    97 => Armco3(armco::Armco),
    /// Armco5
    98 => Armco5(armco::Armco),
    /// Barrier Long
    104 => BarrierLong(barrier::Barrier),
    /// Barrier Red
    105 => BarrierRed(barrier::Barrier),
    /// Barrier White
    106 => BarrierWhite(barrier::Barrier),
    /// Banner
    112 => Banner(banner::Banner),
    /// Ramp1
    120 => Ramp1(ramp::Ramp),
    /// Ramp2
    121 => Ramp2(ramp::Ramp),
    /// Vehicle SUV
    124 => VehicleSUV(vehicle_suv::VehicleSUV),
    /// Vehicle Van
    125 => VehicleVan(vehicle_van::VehicleVan),
    /// Vehicle Truck
    126 => VehicleTruck(vehicle_truck::VehicleTruck),
    /// Vehicle Ambulance
    127 => VehicleAmbulance(vehicle_ambulance::VehicleAmbulance),
    /// Speed hump 10m
    128 => SpeedHump10M(speed_hump::SpeedHump),
    /// Speed hump 6m
    129 => SpeedHump6M(speed_hump::SpeedHump),
    /// Speed hump 2m
    130 => SpeedHump2M(speed_hump::SpeedHump),
    /// Speed hump 1m
    131 => SpeedHump1M(speed_hump::SpeedHump),
    /// Kerb
    132 => Kerb(kerb::Kerb),
    /// Post
    136 => Post(post::Post),
    /// Marquee
    140 => Marquee(marquee::Marquee),
    /// Bale
    144 => Bale(bale::Bale),
    /// Bin1
    145 => Bin1(bin1::Bin1),
    /// Bin2
    146 => Bin2(bin2::Bin2),
    /// Railing1
    147 => Railing1(railing::Railing),
    /// Railing2
    148 => Railing2(railing::Railing),
    /// Start lights 1
    149 => StartLights1(start_lights::StartLights),
    /// Start lights 2
    150 => StartLights2(start_lights::StartLights),
    /// Start lights 3
    151 => StartLights3(start_lights::StartLights),
    /// Metal Sign
    160 => SignMetal(sign_metal::SignMetal),
    /// ChevronLeft
    164 => ChevronLeft(chevron::Chevron),
    /// ChevronRight
    165 => ChevronRight(chevron::Chevron),
    /// Speed Sign
    168 => SignSpeed(sign_speed::SignSpeed),
    /// Concrete Slab
    172 => ConcreteSlab(concrete::ConcreteSlab),
    /// Concrete Ramp
    173 => ConcreteRamp(concrete::ConcreteRamp),
    /// Concrete Wall
    174 => ConcreteWall(concrete::ConcreteWall),
    /// Concrete Pillar
    175 => ConcretePillar(concrete::ConcretePillar),
    /// Concrete Slab Wall
    176 => ConcreteSlabWall(concrete::ConcreteSlabWall),
    /// Concrete Ramp Wall
    177 => ConcreteRampWall(concrete::ConcreteRampWall),
    /// Concrete Short Slab Wall
    178 => ConcreteShortSlabWall(concrete::ConcreteShortSlabWall),
    /// Concrete Wedge
    179 => ConcreteWedge(concrete::ConcreteWedge),
    /// Start position
    184 => StartPosition(start_position::StartPosition),
    /// Pit Startpoint
    185 => PitStartPoint(pit_start_point::PitStartPoint),
    /// Pit stop box
    186 => PitStopBox(pit::PitStopBox),
}
