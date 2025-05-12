#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod angvel;
pub mod decode;
pub mod direction;
pub mod encode;
pub mod game_version;
pub mod license;
pub mod point;
pub mod speed;
pub mod string;
pub mod track;
pub mod vehicle;
pub mod wind;
pub use decode::{Decode, DecodeError, DecodeString};
pub use encode::{Encode, EncodeError, EncodeString};
pub use insim_macros::{Decode, Encode};
