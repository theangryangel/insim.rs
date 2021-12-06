//! Utilities for working with various "custom" strings from Insim.

/// Strip any trailing \0 bytes from a u8 slice.
pub fn strip_trailing_nul(input: &[u8]) -> &[u8] {
    if let Some(rpos) = input.iter().rposition(|x| *x != 0) {
        &input[..=rpos]
    } else {
        input
    }
}

/// Unescape a u8 slice according to LFS' rules.
pub fn unescape(_input: &[u8]) -> &[u8] {
    unimplemented!()
}

mod code_page_string;
mod i_string;

pub use code_page_string::ICodepageString;
pub use i_string::IString;
