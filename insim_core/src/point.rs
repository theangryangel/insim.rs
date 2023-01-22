pub trait Pointable: Copy + Clone + Default {}

impl Pointable for i32 {}
impl Pointable for f32 {}
impl Pointable for u16 {}

// TODO move Point<i32, etc.> here if we can solve the serde issue more gracefully
// without resorting to https://serde.rs/remote-derive.html
