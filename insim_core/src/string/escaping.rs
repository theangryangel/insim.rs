//! Tools to help unescape and escape strings

use std::borrow::Cow;

use super::{colours::ColourMarker, control::ControlMarker};

trait EscapeMarker {
    fn try_lfs_escape(self) -> Option<char>;
    fn try_lfs_unescape(self) -> Option<char>;
}

impl EscapeMarker for char {
    fn try_lfs_unescape(self) -> Option<char> {
        if self.is_lfs_control_char() {
            return Some(char::lfs_control_char());
        }

        match self {
            'v' => Some('|'),
            'a' => Some('*'),
            'c' => Some(':'),
            'd' => Some('\\'),
            's' => Some('/'),
            'q' => Some('?'),
            't' => Some('"'),
            'l' => Some('<'),
            'r' => Some('>'),
            'h' => Some('#'),
            _ => None,
        }
    }

    fn try_lfs_escape(self) -> Option<char> {
        if self.is_lfs_control_char() {
            return Some(char::lfs_control_char());
        }

        match self {
            '|' => Some('v'),
            '*' => Some('a'),
            ':' => Some('c'),
            '\\' => Some('d'),
            '/' => Some('s'),
            '?' => Some('q'),
            '"' => Some('t'),
            '<' => Some('l'),
            '>' => Some('r'),
            '#' => Some('h'),
            _ => None,
        }
    }
}

/// Unescape a u8 slice according to LFS' rules.
pub fn unescape(input: &'_ str) -> Cow<'_, str> {
    // do we need to unescape?
    if !input.chars().any(|c| c.is_lfs_control_char()) {
        return input.into();
    }

    let mut output = String::new();
    let mut chars = input.chars().peekable();

    while let Some(i) = chars.next() {
        if i.is_lfs_control_char()
            && let Some(j) = chars.peek()
            && let Some(k) = j.try_lfs_unescape()
        {
            output.push(k);
            let _ = chars.next(); // advance the iter
        } else {
            output.push(i);
        }
    }

    output.into()
}

/// Unescape a string
pub fn escape(input: &'_ str) -> Cow<'_, str> {
    if !input.chars().any(|c| c.try_lfs_escape().is_some()) {
        return input.into();
    }

    let mut output = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        // is the current char a marker? and do we have a follow up character?
        if c.is_lfs_control_char()
            && let Some(d) = chars.peek()
            && d.is_lfs_colour()
            && let Some(d) = chars.next()
        {
            // is this a colour?
            // just push the colour and move on
            output.push(c);
            output.push(d);
            continue;
        }

        // do we have a character that needs escaping?
        if let Some(d) = c.try_lfs_escape() {
            output.push(char::lfs_control_char());
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
