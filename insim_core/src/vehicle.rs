//! Strongly typed Vehicles for both standard and mods
use std::{borrow::Cow, convert::Infallible, str::FromStr};

use crate::{Decode, Encode, license::License};

macro_rules! define_vehicles {
    (
        $(
            $variant:ident, // Enum Variant (e.g., Xfg)
            $code:literal,  // String Code (e.g., "XFG")
            $license:ident  // License Enum (e.g., Demo)
        ),* $(,)?
    ) => {
        /// Vehicle identifier for standard cars and mods.
        ///
        /// - Standard vehicles use 3-character codes (e.g., `XFG`).
        /// - Mods are encoded as 6-character hex ids.
        /// - `license()` reports the required content tier.
        ///
        /// See <https://www.lfs.net/forum/thread/95662-New-InSim-packet-size-byte-and-mod-info>
        #[derive(PartialEq, Eq, Clone, Copy, Hash)]
        #[non_exhaustive]
        #[allow(missing_docs)]
        pub enum Vehicle {
            $($variant,)*

            Mod(u32),

            /// Unknown vehicle. *Probably* a private mod?
            Unknown,
        }

        impl Vehicle {
            /// This is a built-in vehicle?
            pub fn is_builtin(&self) -> bool {
                !matches!(self, Vehicle::Mod(_))
            }

            /// Is this a user modification?
            pub fn is_mod(&self) -> bool {
                matches!(self, Vehicle::Mod(_))
            }

            /// What content tier is required for this vehicle?
            pub fn license(&self) -> License {
                match self {
                    $(Vehicle::$variant => License::$license,)*
                    Vehicle::Mod(_) => License::S3,
                    Vehicle::Unknown => License::S3,
                }
            }

            /// The shortcode for this vehicle.
            fn code(&self) -> Cow<'static, str> {
                match self {
                    $(Vehicle::$variant => Cow::Borrowed($code),)*
                    Vehicle::Mod(vehmod) => {
                        // Determine the mod id. This is only applicable for Insim v9+.
                        Cow::Owned(format!("{:06X}", vehmod))
                    },
                    Vehicle::Unknown => Cow::Borrowed("Unknown"),
                }
            }
        }

        impl Decode for Vehicle {
            fn decode(buf: &mut bytes::Bytes) -> Result<Self, crate::DecodeError> {
                let mut bytes = buf.split_to(4);
                // conventionally builtins are 3 ascii characters with a \0.
                // so we can take our string input and use this.
                let is_builtin = bytes[0..=2].iter().all(|c| c.is_ascii_alphanumeric()) && bytes[3] == 0;

                match (bytes.as_ref(), is_builtin) {
                    (b"\0\0\0\0", _) => Ok(Vehicle::Unknown),
                    $((value, true) if &value[..3] == $code.as_bytes() => Ok(Vehicle::$variant),)*
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
                    $(Vehicle::$variant => {
                        buf.extend_from_slice($code.as_bytes());
                        ::bytes::BufMut::put_u8(buf, 0);
                    },)*
                    Vehicle::Mod(vehmod) => vehmod.encode(buf)?,
                    Vehicle::Unknown => ::bytes::BufMut::put_bytes(buf, 0, 4),
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
            // Unknown vehicles are always Vehicle::Unknown
            type Err = Infallible;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    $($code => Ok(Self::$variant),)*
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
    };
}

#[rustfmt::skip]
define_vehicles!(
    Xfg, "XFG", Demo,
    Xrg, "XRG", Demo,
    Fbm, "FBM", Demo,

    Xrt, "XRT", S1,
    Rb4, "RB4", S1,
    Fxo, "FXO", S1,
    Lx4, "LX4", S1,
    Lx6, "LX6", S1,
    Mrt, "MRT", S1,

    Uf1, "UF1", S2,
    Rac, "RAC", S2,
    Fz5, "FZ5", S2,
    Fox, "FOX", S2,
    Xfr, "XFR", S2,
    Ufr, "UFR", S2,
    Fo8, "FO8", S2,
    Fxr, "FXR", S2,
    Xrr, "XRR", S2,
    Fzr, "FZR", S2,
    Bf1, "BF1", S2,
);

#[allow(clippy::derivable_impls)]
impl Default for Vehicle {
    fn default() -> Self {
        Vehicle::Xfg
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
    use bytes::{Bytes, BytesMut};

    use super::*;
    use crate::{Decode, Encode};

    #[test]
    fn test_xrt() {
        let v = Vehicle::from_str("XRT").expect("Expected to handle XRT");
        assert_eq!(v, Vehicle::Xrt);
        assert_eq!("XRT", v.to_string());

        let raw = b"XRT\0";
        let mut decode_buf = Bytes::copy_from_slice(raw);
        let decoded = Vehicle::decode(&mut decode_buf).expect("Expected to decode XRT");
        assert_eq!(decoded, Vehicle::Xrt);

        let mut encode_buf = BytesMut::new();
        decoded
            .encode(&mut encode_buf)
            .expect("Expected to encode XRT");
        assert_eq!(encode_buf.as_ref(), raw);
    }

    #[test]
    fn test_fraud_wheeler_e2_rx_manual() {
        let mod_id = 7504921;

        let v = Vehicle::from_str("728419").expect("Expected to parse Fraud Wheeler E2 RX Manual");
        assert_eq!(v, Vehicle::Mod(mod_id));

        let raw = 7_504_921_u32.to_le_bytes();
        let mut decode_buf = Bytes::copy_from_slice(&raw);
        let decoded = Vehicle::decode(&mut decode_buf)
            .expect("Expected to decode Fraud Wheeler E2 RX Manual");
        assert_eq!(decoded, Vehicle::Mod(mod_id));

        let mut encode_buf = bytes::BytesMut::new();
        decoded
            .encode(&mut encode_buf)
            .expect("Expected to encode Fraud Wheeler E2 RX Manual");
        assert_eq!(encode_buf.as_ref(), raw.as_slice());
    }

    #[test]
    fn test_piran_firefly_200() {
        let mod_id = 4301472;
        let v = Vehicle::from_str("41A2A0").expect("Expected to parse Piran Firefly 200");
        assert_eq!(v, Vehicle::Mod(mod_id));

        let raw = 4_301_472_u32.to_le_bytes();
        let mut decode_buf = Bytes::copy_from_slice(&raw);
        let decoded =
            Vehicle::decode(&mut decode_buf).expect("Expected to decode Piran Firefly 200");
        assert_eq!(decoded, Vehicle::Mod(mod_id));

        let mut encode_buf = BytesMut::new();
        decoded
            .encode(&mut encode_buf)
            .expect("Expected to encode Piran Firefly 200");
        assert_eq!(encode_buf.as_ref(), raw.as_slice());
    }

    #[test]
    fn test_unknown() {
        let v = Vehicle::from_str("").expect("Expected to parse Unknown");
        assert_eq!(v, Vehicle::Unknown);

        let raw = b"\0\0\0\0";
        let mut decode_buf = Bytes::copy_from_slice(raw);
        let decoded = Vehicle::decode(&mut decode_buf).expect("Expected to decode Unknown");
        assert_eq!(decoded, Vehicle::Unknown);

        let mut encode_buf = BytesMut::new();
        decoded
            .encode(&mut encode_buf)
            .expect("Expected to encode Unknown");
        assert_eq!(encode_buf.as_ref(), raw);
    }
}
