#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]

pub mod angvel;
pub mod coordinate;
pub mod dash_lights;
pub mod decode;
pub mod encode;
pub mod game_version;
pub mod gear;
pub mod heading;
pub mod identifiers;
pub mod license;
pub mod object;
pub mod speed;
pub mod string;
pub mod track;
pub mod vector;
pub mod vehicle;
pub mod wind;
pub use decode::{Decode, DecodeError, DecodeErrorKind, DecodeString};
pub use encode::{Encode, EncodeError, EncodeErrorKind, EncodeString};
pub use insim_macros::{Decode, Encode};
