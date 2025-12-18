//! Objects are used in both insim and lyt files

pub mod armco;
pub mod bale;
pub mod banner;
pub mod barrier;
pub mod bin;
pub mod chalk;
pub mod concrete;
pub mod cone;
pub mod control;
pub mod insim;
pub mod kerb;
pub mod letterboard;
pub mod marker;
pub mod marquee;
pub mod marshal;
pub mod painted;
pub mod pit;
pub mod post;
pub mod railing;
pub mod ramp;
pub mod sign;
pub mod speed_hump;
pub mod start_lights;
pub mod start_position;
pub mod tyre;
pub mod vehicle;

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

    /// Chalk
    Chalk(chalk::Chalk),
    /// Painted Letters
    PaintLetters(painted::Letters),
    /// Painted Arrows
    PaintArrows(painted::Arrows),
    /// Cones
    Cone(cone::Cone),
    /// Tyres
    TyreStack(tyre::TyreStack),
    /// Corner Marker
    MarkerCorner(marker::MarkerCorner),
    /// Distance Marker
    MarkerDistance(marker::MarkerDistance),
    /// Letterboard
    Letterboard(letterboard::Letterboard),
    /// Armco
    Armco(armco::Armco),
    /// Barriers
    Barrier(barrier::Barrier),
    /// Banner
    Banner(banner::Banner),
    /// Ramp
    Ramp(ramp::Ramp),
    /// Vehicle
    Veh(vehicle::Vehicle),
    /// Speed hump
    SpeedHump(speed_hump::SpeedHump),
    /// Kerb
    Kerb(kerb::Kerb),
    /// Post
    Post(post::Post),
    /// Marquee
    Marquee(marquee::Marquee),
    /// Bale
    Bale(bale::Bale),
    /// Bin1 + Bin2
    Bin(bin::Bin),
    /// Railings
    Railing(railing::Railing),
    /// Start lights 1-3
    StartLights(start_lights::StartLights),
    /// Metal sign, Chevron Left, Chevron Right, Speed
    Sign(sign::Sign),
    /// Concrete
    Concrete(concrete::Concrete),
    /// Start position
    StartPosition(start_position::StartPosition),
    /// Pit Startpoint + box
    Pit(pit::Pit),
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

            4..=13 => ObjectKind::Chalk(chalk::Chalk::from_wire(wire)?),
            16 => ObjectKind::PaintLetters(painted::Letters::from_wire(wire)?),
            17 => ObjectKind::PaintArrows(painted::Arrows::from_wire(wire)?),
            20..=21 | 32..=32 | 40 => ObjectKind::Cone(cone::Cone::from_wire(wire)?),

            48..=55 => ObjectKind::TyreStack(tyre::TyreStack::from_wire(wire)?),

            62 => ObjectKind::MarkerCorner(marker::MarkerCorner::from_wire(wire)?),
            84 => ObjectKind::MarkerDistance(marker::MarkerDistance::from_wire(wire)?),
            92..=93 => ObjectKind::Letterboard(letterboard::Letterboard::from_wire(wire)?),
            96..=98 => ObjectKind::Armco(armco::Armco::from_wire(wire)?),
            104..=106 => ObjectKind::Barrier(barrier::Barrier::from_wire(wire)?),
            112 => ObjectKind::Banner(banner::Banner::from_wire(wire)?),
            120..=121 => ObjectKind::Ramp(ramp::Ramp::from_wire(wire)?),
            124..=127 => ObjectKind::Veh(vehicle::Vehicle::from_wire(wire)?),
            128..=131 => ObjectKind::SpeedHump(speed_hump::SpeedHump::from_wire(wire)?),
            132 => ObjectKind::Kerb(kerb::Kerb::from_wire(wire)?),
            136 => ObjectKind::Post(post::Post::from_wire(wire)?),
            140 => ObjectKind::Marquee(marquee::Marquee::from_wire(wire)?),
            144 => ObjectKind::Bale(bale::Bale::from_wire(wire)?),
            145..=146 => ObjectKind::Bin(bin::Bin::from_wire(wire)?),
            147..=148 => ObjectKind::Railing(railing::Railing::from_wire(wire)?),
            149..=151 => ObjectKind::StartLights(start_lights::StartLights::from_wire(wire)?),
            160 | 168 => ObjectKind::Sign(sign::Sign::from_wire(wire)?),
            172..=179 => ObjectKind::Concrete(concrete::Concrete::from_wire(wire)?),
            184 => ObjectKind::StartPosition(start_position::StartPosition::from_wire(wire)?),
            185..=186 => ObjectKind::Pit(pit::Pit::from_wire(wire)?),

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
            ObjectKind::Control(control) => {
                let mut wire = control.encode()?;
                wire.index = 0;
                wire
            },
            ObjectKind::Marshal(marshal) => {
                let mut wire = marshal.encode()?;
                wire.index = 240;
                wire
            },
            ObjectKind::InsimCheckpoint(insim_checkpoint) => {
                let mut wire = insim_checkpoint.encode()?;
                wire.index = 252;
                wire
            },
            ObjectKind::InsimCircle(insim_circle) => {
                let mut wire = insim_circle.encode()?;
                wire.index = 253;
                wire
            },
            ObjectKind::RestrictedArea(restricted_area) => {
                let mut wire = restricted_area.encode()?;
                wire.index = 254;
                wire
            },
            ObjectKind::RouteChecker(route_checker) => {
                let mut wire = route_checker.encode()?;
                wire.index = 255;
                wire
            },
            ObjectKind::Chalk(chalk) => chalk.to_wire()?,
            ObjectKind::PaintLetters(letters) => letters.to_wire()?,
            ObjectKind::PaintArrows(arrows) => arrows.to_wire()?,
            ObjectKind::Cone(cone) => cone.to_wire()?,
            ObjectKind::TyreStack(tyre_stack) => tyre_stack.to_wire()?,
            ObjectKind::MarkerCorner(marker_corner) => marker_corner.to_wire()?,
            ObjectKind::MarkerDistance(marker_distance) => marker_distance.to_wire()?,
            ObjectKind::Letterboard(letterboard) => letterboard.to_wire()?,
            ObjectKind::Armco(armco) => armco.to_wire()?,
            ObjectKind::Barrier(barrier) => barrier.to_wire()?,
            ObjectKind::Banner(banner) => banner.to_wire()?,
            ObjectKind::Ramp(ramp) => ramp.to_wire()?,
            ObjectKind::Veh(veh) => veh.to_wire()?,
            ObjectKind::SpeedHump(speed_hump) => speed_hump.to_wire()?,
            ObjectKind::Kerb(kerb) => kerb.to_wire()?,
            ObjectKind::Post(post) => post.to_wire()?,
            ObjectKind::Marquee(marquee) => marquee.to_wire()?,
            ObjectKind::Bale(bale) => bale.to_wire()?,
            ObjectKind::Bin(bin) => bin.to_wire()?,
            ObjectKind::Railing(railing) => railing.to_wire()?,
            ObjectKind::StartLights(start_lights) => start_lights.to_wire()?,
            ObjectKind::Sign(sign) => sign.to_wire()?,
            ObjectKind::Concrete(concrete) => concrete.to_wire()?,
            ObjectKind::StartPosition(start_position) => start_position.to_wire()?,
            ObjectKind::Pit(pit) => pit.to_wire()?,
        };
        wire.flags.encode(buf)?;
        wire.index.encode(buf)?;
        wire.heading.encode(buf)?;

        Ok(())
    }
}
