#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]

pub mod angvel;
pub mod dash_lights;
pub mod decode;
pub mod direction;
pub mod encode;
pub mod game_version;
pub mod gear;
pub mod identifiers;
pub mod license;
pub mod object;
pub mod coordinate;
pub mod speed;
pub mod string;
pub mod track;
pub mod vehicle;
pub mod wind;
pub mod vector;
pub use decode::{Decode, DecodeError, DecodeString};
pub use encode::{Encode, EncodeError, EncodeString};
pub use insim_macros::{Decode, Encode};
