//! Strongly typed Vehicles for both standard and mods
use std::{convert::Infallible, str::FromStr};

use crate::{Decode, Encode, license::License};

/// Handles parsing a vehicle name according to the Insim v9 rules.
/// See <https://www.lfs.net/forum/thread/95662-New-InSim-packet-size-byte-and-mod-info>
#[derive(PartialEq, Eq, Clone, Copy, Default, Hash)]
#[non_exhaustive]
#[allow(missing_docs)]
pub enum Vehicle {
    #[default]
    Xfg,
    Xrg,
    Fbm,

    Xrt,
    Rb4,
    Fxo,
    Lx4,
    Lx6,
    Mrt,

    Uf1,
    Rac,
    Fz5,
    Fox,
    Xfr,
    Ufr,
    Fo8,
    Fxr,
    Xrr,
    Fzr,
    Bf1,

    Mod(u32),

    /// Unknown vehicle. *Probably* a private mod?
    Unknown,
}

#[allow(missing_docs)]
impl Vehicle {
    pub fn is_builtin(&self) -> bool {
        !matches!(self, Vehicle::Mod(_))
    }

    pub fn is_mod(&self) -> bool {
        matches!(self, Vehicle::Mod(_))
    }

    pub fn license(&self) -> License {
        match self {
            Vehicle::Xfg => License::Demo,
            Vehicle::Xrg => License::Demo,
            Vehicle::Fbm => License::Demo,

            Vehicle::Xrt => License::S1,
            Vehicle::Rb4 => License::S1,
            Vehicle::Fxo => License::S1,
            Vehicle::Lx4 => License::S1,
            Vehicle::Lx6 => License::S1,
            Vehicle::Mrt => License::S1,

            Vehicle::Uf1 => License::S2,
            Vehicle::Rac => License::S2,
            Vehicle::Fz5 => License::S2,
            Vehicle::Fox => License::S2,
            Vehicle::Xfr => License::S2,
            Vehicle::Ufr => License::S2,
            Vehicle::Fo8 => License::S2,
            Vehicle::Fxr => License::S2,
            Vehicle::Xrr => License::S2,
            Vehicle::Fzr => License::S2,
            Vehicle::Bf1 => License::S2,
            Vehicle::Mod(_) => License::S3,
            Vehicle::Unknown => License::S3,
        }
    }

    pub fn code(&self) -> String {
        match self {
            Vehicle::Xfg => "XFG".to_string(),
            Vehicle::Xrg => "XRG".to_string(),
            Vehicle::Fbm => "FBM".to_string(),
            Vehicle::Xrt => "XRT".to_string(),
            Vehicle::Rb4 => "RB4".to_string(),
            Vehicle::Fxo => "FXO".to_string(),
            Vehicle::Lx4 => "LX4".to_string(),
            Vehicle::Lx6 => "LX6".to_string(),
            Vehicle::Mrt => "MRT".to_string(),
            Vehicle::Uf1 => "UF1".to_string(),
            Vehicle::Rac => "RAC".to_string(),
            Vehicle::Fz5 => "FZ5".to_string(),
            Vehicle::Fox => "FOX".to_string(),
            Vehicle::Xfr => "XFR".to_string(),
            Vehicle::Ufr => "UFR".to_string(),
            Vehicle::Fo8 => "FO8".to_string(),
            Vehicle::Fxr => "FXR".to_string(),
            Vehicle::Xrr => "XRR".to_string(),
            Vehicle::Fzr => "FZR".to_string(),
            Vehicle::Bf1 => "BF1".to_string(),
            Vehicle::Mod(vehmod) => {
                // Determine the mod id. This is only applicable for Insim v9+.
                format!("{:06X}", vehmod)
            },
            Vehicle::Unknown => "Unknown".to_string(),
        }
    }
}

impl Decode for Vehicle {
    fn decode(buf: &mut bytes::Bytes) -> Result<Self, crate::DecodeError> {
        let mut bytes = buf.split_to(4);
        let is_builtin = bytes[0..=2].iter().all(|c| c.is_ascii_alphanumeric()) && bytes[3] == 0;

        match (bytes.as_ref(), is_builtin) {
            ([0, 0, 0, 0], _) => Ok(Vehicle::Unknown),
            ([b'X', b'F', b'G', 0], true) => Ok(Vehicle::Xfg),
            ([b'X', b'R', b'G', 0], true) => Ok(Vehicle::Xrg),
            ([b'F', b'B', b'M', 0], true) => Ok(Vehicle::Fbm),
            ([b'X', b'R', b'T', 0], true) => Ok(Vehicle::Xrt),
            ([b'R', b'B', b'4', 0], true) => Ok(Vehicle::Rb4),
            ([b'F', b'X', b'O', 0], true) => Ok(Vehicle::Fxo),
            ([b'L', b'X', b'4', 0], true) => Ok(Vehicle::Lx4),
            ([b'L', b'X', b'6', 0], true) => Ok(Vehicle::Lx6),
            ([b'M', b'R', b'T', 0], true) => Ok(Vehicle::Mrt),
            ([b'U', b'F', b'1', 0], true) => Ok(Vehicle::Uf1),
            ([b'R', b'A', b'C', 0], true) => Ok(Vehicle::Rac),
            ([b'F', b'Z', b'5', 0], true) => Ok(Vehicle::Fz5),
            ([b'F', b'O', b'X', 0], true) => Ok(Vehicle::Fox),
            ([b'X', b'F', b'R', 0], true) => Ok(Vehicle::Xfr),
            ([b'U', b'F', b'R', 0], true) => Ok(Vehicle::Ufr),
            ([b'F', b'O', b'8', 0], true) => Ok(Vehicle::Fo8),
            ([b'F', b'X', b'R', 0], true) => Ok(Vehicle::Fxr),
            ([b'X', b'R', b'R', 0], true) => Ok(Vehicle::Xrr),
            ([b'F', b'Z', b'R', 0], true) => Ok(Vehicle::Fzr),
            ([b'B', b'F', b'1', 0], true) => Ok(Vehicle::Bf1),
            (_, true) => Err(crate::DecodeErrorKind::BadMagic {
                found: Box::new(bytes),
            }
            .into()),
            (_, false) => Ok(Vehicle::Mod(u32::decode(&mut bytes)?)),
        }
    }
}

impl Encode for Vehicle {
    fn encode(&self, buf: &mut bytes::BytesMut) -> Result<(), crate::EncodeError> {
        match self {
            Vehicle::Xfg => buf.extend_from_slice(&[b'X', b'F', b'G', 0]),
            Vehicle::Xrg => buf.extend_from_slice(&[b'X', b'R', b'G', 0]),
            Vehicle::Fbm => buf.extend_from_slice(&[b'F', b'B', b'M', 0]),
            Vehicle::Xrt => buf.extend_from_slice(&[b'X', b'R', b'T', 0]),
            Vehicle::Rb4 => buf.extend_from_slice(&[b'R', b'B', b'4', 0]),
            Vehicle::Fxo => buf.extend_from_slice(&[b'F', b'X', b'O', 0]),
            Vehicle::Lx4 => buf.extend_from_slice(&[b'L', b'X', b'4', 0]),
            Vehicle::Lx6 => buf.extend_from_slice(&[b'L', b'X', b'6', 0]),
            Vehicle::Mrt => buf.extend_from_slice(&[b'M', b'R', b'T', 0]),
            Vehicle::Uf1 => buf.extend_from_slice(&[b'U', b'F', b'1', 0]),
            Vehicle::Rac => buf.extend_from_slice(&[b'R', b'A', b'C', 0]),
            Vehicle::Fz5 => buf.extend_from_slice(&[b'F', b'Z', b'5', 0]),
            Vehicle::Fox => buf.extend_from_slice(&[b'F', b'O', b'X', 0]),
            Vehicle::Xfr => buf.extend_from_slice(&[b'X', b'F', b'R', 0]),
            Vehicle::Ufr => buf.extend_from_slice(&[b'U', b'F', b'R', 0]),
            Vehicle::Fo8 => buf.extend_from_slice(&[b'F', b'O', b'8', 0]),
            Vehicle::Fxr => buf.extend_from_slice(&[b'F', b'X', b'R', 0]),
            Vehicle::Xrr => buf.extend_from_slice(&[b'X', b'R', b'R', 0]),
            Vehicle::Fzr => buf.extend_from_slice(&[b'F', b'Z', b'R', 0]),
            Vehicle::Bf1 => buf.extend_from_slice(&[b'B', b'F', b'1', 0]),
            Vehicle::Mod(vehmod) => vehmod.encode(buf)?,
            Vehicle::Unknown => buf.extend_from_slice(&[0_u8, 0_u8, 0_u8, 0_u8]),
        };

        Ok(())
    }
}

impl std::fmt::Display for Vehicle {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.code())
    }
}

impl std::fmt::Debug for Vehicle {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.code())
    }
}

impl FromStr for Vehicle {
    // Unknown tracks are always Vehicle::Unknown
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "XFG" => Ok(Self::Xfg),
            "XRG" => Ok(Self::Xrg),
            "FBM" => Ok(Self::Fbm),
            "XRT" => Ok(Self::Xrt),
            "RB4" => Ok(Self::Rb4),
            "FXO" => Ok(Self::Fxo),
            "LX4" => Ok(Self::Lx4),
            "LX6" => Ok(Self::Lx6),
            "MRT" => Ok(Self::Mrt),
            "UF1" => Ok(Self::Uf1),
            "RAC" => Ok(Self::Rac),
            "FZ5" => Ok(Self::Fz5),
            "FOX" => Ok(Self::Fox),
            "XFR" => Ok(Self::Xfr),
            "UFR" => Ok(Self::Ufr),
            "FO8" => Ok(Self::Fo8),
            "FXR" => Ok(Self::Fxr),
            "XRR" => Ok(Self::Xrr),
            "FZR" => Ok(Self::Fzr),
            "BF1" => Ok(Self::Bf1),
            o => {
                if o.len() == 6
                    && let Ok(i) = u32::from_str_radix(o, 16)
                {
                    Ok(Vehicle::Mod(i))
                } else {
                    Ok(Vehicle::Unknown)
                }
            },
        }
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Vehicle {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.collect_str(self)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Vehicle {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        Ok(String::deserialize(deserializer)?
            .parse()
            .expect("Vehicle::FromStr should be infallible"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xrt_from_str() {
        let v = Vehicle::from_str("XRT").expect("Expected to handle XRT");
        assert_eq!(v, Vehicle::Xrt);
        assert_eq!("XRT", v.to_string());
    }

    #[test]
    fn test_mod_from_str() {
        let v = Vehicle::from_str("728419").expect("Expected to handle Mod");
        assert_eq!(v, Vehicle::Mod(7504921));
    }

    #[test]
    fn test_unknown_from_str() {
        let v = Vehicle::from_str("").expect("Expected to handle blank");
        assert_eq!(v, Vehicle::Unknown);
    }
}
