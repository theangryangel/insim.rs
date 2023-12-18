#![doc = include_str!("../README.md")]

pub mod codec;
pub mod connection;
pub mod error;
pub mod insim;
pub mod network;
pub mod packet;
pub mod relay;
pub mod result;

const VERSION: u8 = 9;

#[doc(hidden)]
/// Rexport insim_core
pub use insim_core as core;

#[cfg(feature = "pth")]
/// Report insim_pth when pth feature is enabled
pub use insim_pth as pth;

#[cfg(feature = "smx")]
/// Report insim_smx when smx feature is enabled
pub use insim_smx as smx;
