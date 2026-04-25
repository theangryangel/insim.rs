#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]

#[cfg(test)]
extern crate criterion as _; // Needed for cargo bench. BARF.

pub mod angvel;
#[cfg(feature = "serde")]
pub mod bitflags_serde;
pub mod coordinate;
pub mod dash_lights;
pub mod decode;
pub mod encode;
pub mod game_version;
pub mod gear;
pub mod heading;
pub(crate) mod hex;
pub mod identifiers;
pub mod license;
pub mod object;
pub mod speed;
pub mod string;
pub mod track;
pub mod vector;
pub mod vehicle;
pub mod wind;
pub use decode::{Decode, DecodeContext, DecodeError, DecodeErrorKind};
pub use encode::{Encode, EncodeContext, EncodeError, EncodeErrorKind};
pub use insim_macros::{Decode, Encode};
