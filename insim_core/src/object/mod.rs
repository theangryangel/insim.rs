//! Objects are used in both insim and lyt files

// pub mod axo;
// pub mod chalk;
// pub mod concrete;
// pub mod control;
// pub mod tyre;
pub mod marshal;
pub mod insim;

use crate::{Decode, DecodeError, Encode, EncodeError};

trait ObjectVariant: Sized {
    /// Encode this Object, returning (flags, heading)
    fn encode(&self) -> Result<(u8, u8), EncodeError>;
    /// Bytes into an Object
    fn decode(flags: u8, heading: u8) -> Result<Self, DecodeError>;
}

macro_rules! generate_object {
    (
        $(
            $(#[$attr:meta])*
            $index:literal => $name:ident($ty:path),
        )*
    ) => {
        #[derive(Debug, Clone, Default, PartialEq, Eq)]
        #[cfg_attr(feature = "serde", derive(serde::Serialize))]
        pub struct Object {
            /// Object xyz position
            pub xyz: glam::I16Vec3,
            /// Kind
            pub kind: ObjectKind,
        }

        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
        #[cfg_attr(feature = "serde", derive(serde::Serialize))]
        #[non_exhaustive]
        pub enum ObjectKind {
            $(
                $(#[$attr])*
                $name($ty),
            )*
        }

        impl Default for ObjectKind {
            fn default() -> Self { 
                todo!() 
            }
        }

        impl Decode for Object {
            fn decode(buf: &mut bytes::Bytes) -> Result<Self, DecodeError> {
                let x = i16::decode(buf)?;
                let y = i16::decode(buf)?;
                let z = u8::decode(buf)?;

                let flags = u8::decode(buf)?;
                let index = u8::decode(buf)?;
                let heading = u8::decode(buf)?;

                match index {
                    $(
                        $index => {
                            let inner = <$ty>::decode(flags, heading)?;
                            Ok(Self::$name(inner))
                        }
                    )*
                    _ => Err(format!("Unknown object index: {}", index).into()),
                }

                let kind = ObjectKind::decode(flags, index, heading)?;

                Ok(Self {
                    xyz: glam::I16Vec3 { x, y, z: z as i16 },
                    kind,
                })
            }
        }

        impl Encode for Object {
            fn encode(&self, buf: &mut bytes::BytesMut) -> Result<(), EncodeError> {
                self.xyz.x.encode(buf)?;
                self.xyz.y.encode(buf)?;
                (self.xyz.z as u8).encode(buf)?; // FIXME: use TryFrom
                let (index, flags, heading) = match self {
                    $(
                        Self::$name(inner) => {
                            let (flags, heading) = inner.encode()?;
                            ($index, flags, heading)
                        }
                    )*
                };
                flags.encode(buf)?;
                index.encode(buf)?;
                self.heading.encode(buf)?;

                Ok(())
            }
        }

    };
}

generate_object! {
  /// Special control object
  0 => Control(control::Control),

  /// A marshal
  240 => Marshal(marshal::Marshal),

  /// Insim Checkpoint
  252 => InsimCheckpoint(insim::InsimCheckpoint),

  /// Insim circle
  253 => InsimCircle(insim::InsimCircle),

  /// Restrited area / circle
  254 => RestrictedArea(marshal::RestrictedArea),

  /// Route checker
  255 => RouteChecker(marshal::RouteChecker),
}
