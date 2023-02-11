//! Utilities for working with various strings from Insim.

pub const MARKER: u8 = b'^';

mod escaping;
pub use escaping::*;

pub mod codepages;
pub mod colours;
