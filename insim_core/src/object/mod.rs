//! Objects are used in both insim and lyt files

pub mod chalk;
// pub mod concrete;
pub mod cone;
pub mod control;
pub mod insim;
pub mod marshal;
pub mod painted;
pub mod tyre;

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

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[non_exhaustive]
/// Layout Object Kind
pub enum ObjectKind {
    /// Control - start, finish, checkpoints
    Control(control::Control),

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
    MarkerCorner(),

    /// Distance Marker
    MarkerDistance(),

    /// Letterboard
    Letterboard(),

    /// Armco
    Armco(),

    /// Barriers
    Barrier(),

    /// Banner
    Banner(),

    /// Ramp
    Ramp(),

    /// Vehicle
    Veh(),

    /// Speed hump
    SpeedHump(),

    /// Kerb
    Kerb(),

    /// Post
    Post(),

    /// Marquee
    Marquee(),

    /// Bale
    Bale(),

    /// Bin1 + Bin2
    Bin(),

    /// Railings
    Railing(),

    /// Start lights 1-3
    StartLights(),

    /// Metal sign, Chevron Left, Chevron Right, Speed
    Sign(),

    /// Concrete
    Concrete,

    /// Start position
    StartPosition(),

    /// Pit Startpoint + box
    Pit(),

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

            4..=13 => ObjectKind::Chalk(chalk::Chalk::decode(index, flags, heading)?),
            16 => ObjectKind::PaintLetters(painted::Letters::decode(index, flags, heading)?),
            17 => ObjectKind::PaintArrows(painted::Arrows::decode(index, flags, heading)?),
            20..=21 | 32..=32 | 40 => ObjectKind::Cone(cone::Cone::decode(index, flags, heading)?),

            48..=55 => ObjectKind::TyreStack(tyre::TyreStack::decode(index, flags, heading)?),

            62 =>
            /* MarkerCorner */
            {
                todo!()
            },
            84 =>
            /* MarkerDistance */
            {
                todo!()
            },
            92..=93 =>
            /* Letterboard */
            {
                todo!()
            },
            96..=98 =>
            /* Armco */
            {
                todo!()
            },
            104..=106 =>
            /* Barrier */
            {
                todo!()
            },
            112 =>
            /* Banner */
            {
                todo!()
            },
            120..=121 =>
            /* Ramp */
            {
                todo!()
            },
            124..=127 =>
            /* Vehicles */
            {
                todo!()
            },
            128..=131 =>
            /* SpeedHump */
            {
                todo!()
            },
            132 =>
            /* Kerb */
            {
                todo!()
            },
            136 =>
            /* Post */
            {
                todo!()
            },
            140 =>
            /* Marquee */
            {
                todo!()
            },
            144 =>
            /* Bale */
            {
                todo!()
            },
            145..=146 =>
            /* Bin */
            {
                todo!()
            },
            147..=148 =>
            /* Railings */
            {
                todo!()
            },
            149..=151 =>
            /* StartLights */
            {
                todo!()
            },

            160 | 164 | 165 | 168 =>
            /* Signs */
            {
                todo!()
            },
            172..=179 =>
            /* Concrete */
            {
                todo!()
            },

            184 =>
            /* Start Position */
            {
                todo!()
            },
            185 | 186 =>
            /* Pit */
            {
                todo!()
            },

            240 => ObjectKind::Marshal(marshal::Marshal::decode(flags, heading)?),
            252 => ObjectKind::InsimCheckpoint(insim::InsimCheckpoint::decode(flags, heading)?),
            253 => ObjectKind::InsimCircle(insim::InsimCircle::decode(flags, heading)?),
            254 => ObjectKind::RestrictedArea(marshal::RestrictedArea::decode(flags, heading)?),
            255 => ObjectKind::RouteChecker(marshal::RouteChecker::decode(flags, heading)?),

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
            ObjectKind::Chalk(chalk) => chalk.encode()?,
            ObjectKind::PaintLetters(letters) => letters.encode()?,
            ObjectKind::PaintArrows(arrows) => arrows.encode()?,
            ObjectKind::Cone(cone) => cone.encode()?,
            ObjectKind::TyreStack(tyre_stack) => tyre_stack.encode()?,
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
            ObjectKind::MarkerCorner() => todo!(),
            ObjectKind::MarkerDistance() => todo!(),
            ObjectKind::Letterboard() => todo!(),
            ObjectKind::Armco() => todo!(),
            ObjectKind::Barrier() => todo!(),
            ObjectKind::Banner() => todo!(),
            ObjectKind::Ramp() => todo!(),
            ObjectKind::Veh() => todo!(),
            ObjectKind::SpeedHump() => todo!(),
            ObjectKind::Kerb() => todo!(),
            ObjectKind::Post() => todo!(),
            ObjectKind::Marquee() => todo!(),
            ObjectKind::Bale() => todo!(),
            ObjectKind::Bin() => todo!(),
            ObjectKind::Railing() => todo!(),
            ObjectKind::StartLights() => todo!(),
            ObjectKind::Sign() => todo!(),
            ObjectKind::Concrete => todo!(),
            ObjectKind::StartPosition() => todo!(),
            ObjectKind::Pit() => todo!(),
        };
        flags.encode(buf)?;
        index.encode(buf)?;
        heading.encode(buf)?;

        Ok(())
    }
}
