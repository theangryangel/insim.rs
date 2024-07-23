//! Strongly typed wind strength
use binrw::binrw;
#[cfg(feature = "serde")]
use serde::Serialize;

#[derive(Debug, Default, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[repr(u8)]
#[binrw]
#[brw(repr(u8))]
/// Wind strength levels within LFS
pub enum Wind {
    #[default]
    /// No wind
    None = 0,
    /// Weak wind
    Weak = 1,
    /// Strong wind
    Strong = 2,
}
