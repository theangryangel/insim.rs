//! Utilities for working with various strings from Insim.

use if_chain::if_chain;

pub mod codepages;
pub mod colours;

pub const MARKER: u8 = b'^';

pub const ESCAPE_SEQUENCES: &[(u8, u8)] = &[
    (b'v', b'|'),
    (b'a', b'*'),
    (b'c', b':'),
    (b'd', b'\\'),
    (b's', b'/'),
    (b'q', b'?'),
    (b't', b'"'),
    (b'l', b'<'),
    (b'r', b'>'),
    (b'h', b'#'),
    (b'^', b'^'),
];

/// Determine if a u8 can represent an A-Za-z0-9 ASCII character.
pub fn is_ascii_alphanumeric(c: &u8) -> bool {
    // 0-9
    if (30..=57).contains(c) {
        return true;
    }

    // A-Z
    if (65..=90).contains(c) {
        return true;
    }

    // a-z
    if (97..=122).contains(c) {
        return true;
    }

    false
}

/// Strip any trailing \0 bytes from a u8 slice.
pub fn strip_trailing_nul(input: &[u8]) -> &[u8] {
    if let Some(pos) = input.iter().position(|x| *x == 0) {
        &input[..pos]
    } else {
        input
    }
}

// TODO This should also probably take and return a Cow.
/// Unescape a u8 slice according to LFS' rules.
pub fn unescape(input: &str) -> String {
    let mut maybe_needs_unescaping = false;

    for c in input.chars() {
        if c == MARKER as char {
            maybe_needs_unescaping = true;
            break;
        }
    }

    if !maybe_needs_unescaping {
        return input.to_string();
    }

    let mut output = String::new();
    let mut chars = input.chars().peekable();

    while let Some(i) = chars.next() {
        if i == MARKER as char {
            if let Some(j) = chars.peek() {
                if let Some(k) = ESCAPE_SEQUENCES.iter().find(|x| x.0 as char == *j) {
                    output.push(k.1 as char);
                    let _ = chars.next(); // advance the iter
                    continue;
                }
            }
        }

        output.push(i);
    }

    output
}

// TODO: This should probably be a Cow
/// Unescape a string
pub fn escape(input: &str) -> String {
    let mut output = String::new();
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        // is the current char a marker? and do we have a follow up character?
        // TODO: replace with a let chain when its stable
        if_chain! {
            if c == MARKER as char;
            if let Some(d) = chars.peek();
            if colours::COLOUR_SEQUENCES.contains(d);
            then {
                // is this a colour?
                // just push the colour and move on
                output.push(MARKER as char);
                output.push(chars.next().unwrap());
                continue;
            }
        }

        // do we have a character that needs escaping?
        if let Some(i) = ESCAPE_SEQUENCES.iter().find(|i| i.1 as char == c) {
            output.push(MARKER as char);
            output.push(i.0 as char);
            continue;
        }

        output.push(c)
    }

    output
}
