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

trait ObjectVariant: Sized {
    /// Encode this Object, returning (u8, flags, heading)
    fn encode(&self) -> Result<(u8, u8, u8), EncodeError>;
    /// Bytes into an Object
    fn decode(index: u8, flags: u8, heading: u8) -> Result<Self, DecodeError>;
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

        let kind = match index {
            0 => ObjectKind::Control(control::Control::decode(flags, heading)?),
            240 => ObjectKind::Marshal(marshal::Marshal::decode(flags, heading)?),
            252 => ObjectKind::InsimCheckpoint(insim::InsimCheckpoint::decode(flags, heading)?),
            253 => ObjectKind::InsimCircle(insim::InsimCircle::decode(flags, heading)?),
            254 => ObjectKind::RestrictedArea(marshal::RestrictedArea::decode(flags, heading)?),
            255 => ObjectKind::RouteChecker(marshal::RouteChecker::decode(flags, heading)?),

            4..=13 => ObjectKind::Chalk(chalk::Chalk::decode(index, flags, heading)?),
            16 => ObjectKind::PaintLetters(painted::Letters::decode(index, flags, heading)?),
            17 => ObjectKind::PaintArrows(painted::Arrows::decode(index, flags, heading)?),
            20..=21 | 32..=32 | 40 => ObjectKind::Cone(cone::Cone::decode(index, flags, heading)?),

            48..=55 => ObjectKind::TyreStack(tyre::TyreStack::decode(index, flags, heading)?),

            62 => ObjectKind::MarkerCorner(marker::MarkerCorner::decode(index, flags, heading)?),
            84 => {
                ObjectKind::MarkerDistance(marker::MarkerDistance::decode(index, flags, heading)?)
            },
            92..=93 => {
                ObjectKind::Letterboard(letterboard::Letterboard::decode(index, flags, heading)?)
            },
            96..=98 => ObjectKind::Armco(armco::Armco::decode(index, flags, heading)?),
            104..=106 => ObjectKind::Barrier(barrier::Barrier::decode(index, flags, heading)?),
            112 => ObjectKind::Banner(banner::Banner::decode(index, flags, heading)?),
            120..=121 => ObjectKind::Ramp(ramp::Ramp::decode(index, flags, heading)?),
            124..=127 => ObjectKind::Veh(vehicle::Vehicle::decode(index, flags, heading)?),
            128..=131 => {
                ObjectKind::SpeedHump(speed_hump::SpeedHump::decode(index, flags, heading)?)
            },
            132 => ObjectKind::Kerb(kerb::Kerb::decode(index, flags, heading)?),
            136 => ObjectKind::Post(post::Post::decode(index, flags, heading)?),
            140 => ObjectKind::Marquee(marquee::Marquee::decode(index, flags, heading)?),
            144 => ObjectKind::Bale(bale::Bale::decode(index, flags, heading)?),
            145..=146 => ObjectKind::Bin(bin::Bin::decode(index, flags, heading)?),
            147..=148 => ObjectKind::Railing(railing::Railing::decode(index, flags, heading)?),
            149..=151 => {
                ObjectKind::StartLights(start_lights::StartLights::decode(index, flags, heading)?)
            },
            160 | 168 => ObjectKind::Sign(sign::Sign::decode(index, flags, heading)?),
            172..=179 => ObjectKind::Concrete(concrete::Concrete::decode(index, flags, heading)?),
            184 => ObjectKind::StartPosition(start_position::StartPosition::decode(
                index, flags, heading,
            )?),
            185..=186 => ObjectKind::Pit(pit::Pit::decode(index, flags, heading)?),

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
        let (index, flags, heading) = match &self.kind {
            ObjectKind::Control(control) => {
                let (flags, heading) = control.encode()?;
                (0, flags, heading)
            },
            ObjectKind::Marshal(marshal) => {
                let (flags, heading) = marshal.encode()?;
                (240, flags, heading)
            },
            ObjectKind::InsimCheckpoint(insim_checkpoint) => {
                let (flags, heading) = insim_checkpoint.encode()?;
                (252, flags, heading)
            },
            ObjectKind::InsimCircle(insim_circle) => {
                let (flags, heading) = insim_circle.encode()?;
                (253, flags, heading)
            },
            ObjectKind::RestrictedArea(restricted_area) => {
                let (flags, heading) = restricted_area.encode()?;
                (254, flags, heading)
            },
            ObjectKind::RouteChecker(route_checker) => {
                let (flags, heading) = route_checker.encode()?;
                (255, flags, heading)
            },
            ObjectKind::Chalk(chalk) => chalk.encode()?,
            ObjectKind::PaintLetters(letters) => letters.encode()?,
            ObjectKind::PaintArrows(arrows) => arrows.encode()?,
            ObjectKind::Cone(cone) => cone.encode()?,
            ObjectKind::TyreStack(tyre_stack) => tyre_stack.encode()?,
            ObjectKind::MarkerCorner(marker_corner) => marker_corner.encode()?,
            ObjectKind::MarkerDistance(marker_distance) => marker_distance.encode()?,
            ObjectKind::Letterboard(letterboard) => letterboard.encode()?,
            ObjectKind::Armco(armco) => armco.encode()?,
            ObjectKind::Barrier(barrier) => barrier.encode()?,
            ObjectKind::Banner(banner) => banner.encode()?,
            ObjectKind::Ramp(ramp) => ramp.encode()?,
            ObjectKind::Veh(veh) => veh.encode()?,
            ObjectKind::SpeedHump(speed_hump) => speed_hump.encode()?,
            ObjectKind::Kerb(kerb) => kerb.encode()?,
            ObjectKind::Post(post) => post.encode()?,
            ObjectKind::Marquee(marquee) => marquee.encode()?,
            ObjectKind::Bale(bale) => bale.encode()?,
            ObjectKind::Bin(bin) => bin.encode()?,
            ObjectKind::Railing(railing) => railing.encode()?,
            ObjectKind::StartLights(start_lights) => start_lights.encode()?,
            ObjectKind::Sign(sign) => sign.encode()?,
            ObjectKind::Concrete(concrete) => concrete.encode()?,
            ObjectKind::StartPosition(start_position) => start_position.encode()?,
            ObjectKind::Pit(pit) => pit.encode()?,
        };
        flags.encode(buf)?;
        index.encode(buf)?;
        heading.encode(buf)?;

        Ok(())
    }
}
