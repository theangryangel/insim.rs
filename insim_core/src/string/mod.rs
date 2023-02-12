//! Utilities for working with various strings from Insim.

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

/// Unescape a u8 slice according to LFS' rules.
pub fn unescape(input: &[u8]) -> Vec<u8> {
    let mut maybe_needs_unescaping = false;

    for c in input.iter() {
        if *c == b'^' {
            maybe_needs_unescaping = true;
            break;
        }
    }

    if !maybe_needs_unescaping {
        return input.to_vec();
    }

    let mut output = Vec::with_capacity(input.len());
    let mut iter = input.iter().peekable();

    while let Some(i) = iter.next() {
        if *i == MARKER {
            if let Some(j) = iter.peek() {
                if let Some(k) = ESCAPE_SEQUENCES.iter().find(|x| x.0 == **j) {
                    output.push(k.1);
                    let _ = iter.next(); // advance the iter
                    continue;
                }
            }
        }

        output.push(*i);
    }

    output
}
