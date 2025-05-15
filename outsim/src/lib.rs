#![doc = include_str!("../README.md")]
#![cfg_attr(test, deny(warnings, unreachable_pub))]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]

pub mod identifiers;
pub mod outsimpack;
pub mod outsimpack2;
pub use ::insim_core as core;
pub use identifiers::OutsimId;
pub use outsimpack::OutsimPack;
