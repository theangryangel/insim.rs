//! Tools to help unescape and escape strings

use std::borrow::Cow;

use if_chain::if_chain;

use super::{colours, MARKER};

mod mappings {

    /// Special character escape sequences
    /// ^v = |, etc.
    const MAPPINGS: [(char, char); 11] = [
        ('v', '|'),
        ('a', '*'),
        ('c', ':'),
        ('d', '\\'),
        ('s', '/'),
        ('q', '?'),
        ('t', '"'),
        ('l', '<'),
        ('r', '>'),
        ('h', '#'),
        ('^', '^'),
    ];

    pub(super) fn try_unescape(c: char) -> Option<char> {
        MAPPINGS.iter().find(|x| x.0 == c).map(|k| k.1)
    }

    pub(super) fn try_escape(c: char) -> Option<char> {
        MAPPINGS.iter().find(|x| x.1 == c).map(|k| k.0)
    }

    pub(super) fn needs_escaping(input: &str) -> bool {
        for c in input.chars() {
            if MAPPINGS.iter().any(|i| i.1 == c) {
                return true;
            }
        }

        false
    }

    pub(super) fn needs_unescaping(input: &str) -> bool {
        input.chars().any(|c| c == super::MARKER)
    }
}

/// Unescape a u8 slice according to LFS' rules.
pub fn unescape(input: &str) -> Cow<str> {
    if !mappings::needs_unescaping(input) {
        return input.into();
    }

    let mut output = String::new();
    let mut chars = input.chars().peekable();

    while let Some(i) = chars.next() {
        if_chain! {
            if i == MARKER;
            if let Some(j) = chars.peek();
            if let Some(k) = mappings::try_unescape(*j);
            then {
                output.push(k);
                let _ = chars.next(); // advance the iter
            } else {
                output.push(i);
            }
        }
    }

    output.into()
}

/// Unescape a string
pub fn escape(input: &str) -> Cow<str> {
    if !mappings::needs_escaping(input) {
        return input.into();
    }

    let mut output = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        // is the current char a marker? and do we have a follow up character?
        if_chain! {
            if c == MARKER;
            if let Some(d) = chars.peek();
            if colours::COLOUR_SEQUENCES.contains(d);
            then {
                // is this a colour?
                // just push the colour and move on
                output.push(MARKER);
                output.push(chars.next().unwrap());
                continue;
            }
        }

        // do we have a character that needs escaping?
        if let Some(d) = mappings::try_escape(c) {
            output.push(MARKER);
            output.push(d);
            continue;
        }

        output.push(c)
    }

    output.into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escaping_and_unescaping() {
        let original = "^|*:\\/?\"<>#123^945";

        let escaped = escape(original);
        assert_eq!(escaped, "^^^v^a^c^d^s^q^t^l^r^h123^945");

        let unescaped = unescape(&escaped);
        assert_eq!(unescaped, original);
    }
}
