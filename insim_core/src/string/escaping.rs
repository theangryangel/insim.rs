//! Tools to help unescape and escape strings

use std::borrow::Cow;

use if_chain::if_chain;

use super::{colours, MARKER};

/// Special character escape sequences
pub(crate) static ESCAPE_SEQUENCES: [(char, char); 11] = [
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

/// Unescape a u8 slice according to LFS' rules.
pub fn unescape(input: Cow<str>) -> Cow<str> {
    let maybe_needs_unescaping = input.chars().any(|c| c == MARKER);

    if !maybe_needs_unescaping {
        return input;
    }

    let mut output = String::new();
    let mut chars = input.chars().peekable();

    while let Some(i) = chars.next() {
        if i == MARKER {
            if let Some(j) = chars.peek() {
                if let Some(k) = ESCAPE_SEQUENCES.iter().find(|x| x.0 == *j) {
                    output.push(k.1);
                    let _ = chars.next(); // advance the iter
                    continue;
                }
            }
        }

        output.push(i);
    }

    output.into()
}

/// Unescape a string
pub fn escape(input: Cow<str>) -> Cow<str> {
    let mut maybe_needs_unescaping = false;

    // TODO: We can probably do this better
    for c in input.chars() {
        if ESCAPE_SEQUENCES.iter().any(|i| i.1 == c) {
            maybe_needs_unescaping = true;
            break;
        }
    }

    if !maybe_needs_unescaping {
        return input;
    }

    let mut output = String::new();
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        // is the current char a marker? and do we have a follow up character?
        // TODO: replace with a let chain when its stable
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
        if let Some(i) = ESCAPE_SEQUENCES.iter().find(|i| i.1 == c) {
            output.push(MARKER);
            output.push(i.0);
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
        let original: Cow<str> = "^|*:\\/?\"<>#123^945".into();

        let escaped = escape(original.clone());
        let unescaped = unescape(escaped.clone());

        assert_eq!(escaped, "^^^v^a^c^d^s^q^t^l^r^h123^945");
        assert_eq!(unescaped, original);
    }
}
