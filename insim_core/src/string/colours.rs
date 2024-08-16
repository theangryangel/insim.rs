//! Utilities for working with colours from Insim.

use std::borrow::Cow;

use super::MARKER;

/// Supported colour codes within LFS
pub(super) const COLOUR_SEQUENCES: &[char] = &['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];

/// Strip LFS colours
pub fn strip(input: &str) -> Cow<str> {
    if !input.chars().any(|c| c == MARKER) {
        return input.into();
    }

    let mut iter = input.chars().peekable();
    let mut output = String::with_capacity(input.len());

    while let Some(i) = iter.next() {
        match (i, iter.peek()) {
            // Special case, ignore escaped markers (AKA ^^)
            // If we don't do this now, and just fall through the next check, something like ^^1
            // wont be handled correctly!
            (MARKER, Some(&MARKER)) => {
                output.push(MARKER);
                output.push(MARKER);
                let _ = iter.next();
            },

            (MARKER, Some(j)) if COLOUR_SEQUENCES.contains(j) => {
                let _ = iter.next();
            },

            _ => output.push(i),
        }
    }

    Cow::Owned(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_colours_only() {
        assert_eq!(strip("^1^2^3^4^5^6^7^8^9"), "");
    }

    #[test]
    fn test_strip_colours() {
        assert_eq!(strip("^1234^56789"), "2346789");
    }

    #[test]
    fn test_strip_colours_escaped() {
        assert_eq!(strip("^^1234^56789"), "^^12346789");
    }
}
