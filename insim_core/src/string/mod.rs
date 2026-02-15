//! Utilities for working with various strings from Insim.
//!
//! For incoming LFS text, the safest processing order is usually:
//! 1. decode codepages with [`codepages::to_lossy_string`]
//! 2. process colours while the string is still escaped (for example with [`colours::strip`])
//! 3. unescape with [`escaping::unescape`] when you no longer need marker semantics
//!
//! This avoids losing intent around escaped control markers (`^^`).

pub mod codepages;
mod codepages_lut;
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
