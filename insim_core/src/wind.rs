//! Strongly typed wind strength
use binrw::binrw;
#[cfg(feature = "serde")]
use serde::Serialize;

#[derive(Debug, Default, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[repr(u8)]
#[binrw]
#[brw(repr(u8))]
pub enum Wind {
    #[default]
    None = 0,
    Weak = 1,
    Strong = 2,
}
