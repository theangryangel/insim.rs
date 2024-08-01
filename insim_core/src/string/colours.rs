//! Utilities for working with colours from Insim.

use std::borrow::Cow;

use super::MARKER;

/// Supported colour codes within LFS
pub(crate) static COLOUR_SEQUENCES: &[char] = &['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];

/// Strip LFS colours
pub fn strip(input: &str) -> Cow<str> {
    let mut iter = input.chars().peekable();

    if !input.chars().any(|c| c == MARKER) {
        return input.into();
    }

    let mut output = String::with_capacity(input.len());

    while let Some(i) = iter.next() {
        match (i, iter.peek()) {
            // special case escaped ^ we dont want to strip
            (MARKER, Some('^')) => {
                output.push(MARKER);
                output.push(iter.next().unwrap());
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
