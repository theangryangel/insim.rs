/// Control Character is used to identify LFS' escaping and special character
pub(super) trait ControlCharacter {
    /// Builder to return the LFS control character
    fn lfs_control_char() -> Self;
    /// Is this a LFS control character?
    fn is_lfs_control_char(&self) -> bool;
}

impl ControlCharacter for u8 {
    fn lfs_control_char() -> u8 {
        char::lfs_control_char() as u8
    }

    fn is_lfs_control_char(&self) -> bool {
        (*self as char).is_lfs_control_char()
    }
}

impl ControlCharacter for char {
    fn lfs_control_char() -> char {
        '^'
    }

    fn is_lfs_control_char(&self) -> bool {
        *self == '^'
    }
}
