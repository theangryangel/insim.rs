//! Limit

/// Describes the Left and Right limit, of a given node.
#[derive(Debug, Copy, Clone, Default, PartialEq, insim_core::Decode, insim_core::Encode)]
pub struct Limit {
    /// Left track limit
    pub left: f32,

    /// Right track limit
    pub right: f32,
}
