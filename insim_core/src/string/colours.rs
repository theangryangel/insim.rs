//! Utilities for working with colours from Insim.

use super::MARKER;

/// Supported colour codes within LFS
pub const COLOUR_SEQUENCES: &[char] = &['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];

/// Strip LFS colours
pub fn strip(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut iter = input.chars();

    while let Some(i) = iter.next() {
        if i == (MARKER as char) {
            if let Some(j) = iter.next() {
                if COLOUR_SEQUENCES.contains(&j) {
                    continue;
                }

                output.push(i);
                output.push(j);
                continue;
            }
        }

        output.push(i);
    }

    output
}

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
