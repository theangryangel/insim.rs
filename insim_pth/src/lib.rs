//! # insim_pth
//!
//! Parse a Live for Speed pth (path) and pin files.
//!
//! Supports both LFSPTH and SRPATH files.
//!
//! A pth file consists of a series points [`Node`][node::Node], with direction and width ([`Limit`][limit::Limit]),
//! that describe the track that you drive along.
//!
//! Historically LFS has used the PTH to watch your progress along the track, decides
//! if you are driving in reverse, the yellow and blue flag systems, the position list,
//! timing, etc.
//!
//! On a standard LFS track the [`Node`][node::Node] is communicated via MCI and NLP Insim packets.
//!
//! On an open configuration [`Node`][node::Node] are not used and are unavailable via Insim MCI packets.
//!
//! The distance between each [`Node`][node::Node] is not constant. According to the LFS developers
//! there is approximately 0.2 seconds of time between passing one node and the next,
//! when you are "driving at a reasonable speed".
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]

pub mod pin;
pub mod pth;

pub mod error;
pub mod limit;
pub mod node;

pub use error::Error;
pub use pin::Pin;
pub use pth::Pth;
