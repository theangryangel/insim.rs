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
    if let Some(rpos) = input.iter().rposition(|x| *x != 0) {
        &input[..=rpos]
    } else {
        input
    }
}

const ESCAPE_SEQUENCES: &[(u8, u8)] = &[
    (b'v', b'|'),
    (b'a', b'*'),
    (b'c', b':'),
    (b'd', b'\\'),
    (b's', b'/'),
    (b'q', b'?'),
    (b't', b'"'),
    (b'l', b'<'),
    (b'r', b'>'),
];

/// Escape a u8 slice according to LFS' rules.
// FIXME: Doesn't escape ^ when it needs to be.
// Temporarily allow_let_on_iterator as I'll be needing peek (probably) for handling the escape of
// '^'
#[allow(clippy::while_let_on_iterator)]
pub fn escape(input: &[u8]) -> Vec<u8> {
    // FIXME: probably should make this a Cow?

    let mut output = Vec::with_capacity(input.len());
    let mut iter = input.iter();

    while let Some(i) = iter.next() {
        if let Some(k) = ESCAPE_SEQUENCES.iter().find(|x| x.1 == *i) {
            output.push(b'^');
            output.push(k.0);
            continue;
        }

        output.push(*i);
    }

    output
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
    let mut iter = input.iter();

    while let Some(i) = iter.next() {
        if *i == b'^' {
            if let Some(j) = iter.next() {
                if let Some(k) = ESCAPE_SEQUENCES.iter().find(|x| x.0 == *j) {
                    output.push(k.1);
                } else {
                    output.push(*i);
                    output.push(*j);
                }
                continue;
            }
        }

        output.push(*i);
    }

    output
}
