//! Utilities for working with various strings from Insim.

pub mod codepages;
pub mod colours;
mod control;
pub mod escaping;

/// Strip any trailing \0 bytes from a u8 slice.
pub fn strip_trailing_nul(input: &[u8]) -> &[u8] {
    if let Some(pos) = input.iter().position(|x| *x == 0) {
        &input[..pos]
    } else {
        input
    }
}
