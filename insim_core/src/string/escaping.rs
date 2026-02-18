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

/// Trait to help escape and unescape strings.
pub trait Escape {
    /// Escape a string according to LFS' rules.
    fn escape(&self) -> Cow<'_, str>;
    /// Unescape a string according to LFS' rules.
    ///
    /// This is lossy with respect to control-marker intent. For example, `^^0` becomes `^0`.
    /// If you need colour-aware handling, process colours before unescaping.
    fn unescape(&self) -> Cow<'_, str>;
}

impl<T: AsRef<str>> Escape for T {
    fn escape(&self) -> Cow<'_, str> {
        crate::string::escaping::escape(self.as_ref())
    }

    fn unescape(&self) -> Cow<'_, str> {
        #[allow(deprecated)]
        crate::string::escaping::unescape(self.as_ref())
    }
}

/// Unescape a string according to LFS' rules.
///
/// This is lossy with respect to control-marker intent. For example, `^^0` becomes `^0`.
/// If you need colour-aware handling, process colours before unescaping.
/// Prefer using the [`Escape::unescape`] trait function.
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
/// Prefer using the [`Escape::escape`] trait function.
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
        let original = "^|*:\\/?\"<>#123^845";

        let escaped = escape(original);
        assert_eq!(escaped, "^^^v^a^c^d^s^q^t^l^r^h123^845");

        let unescaped = unescape(&escaped);
        assert_eq!(unescaped, original);
    }

    #[test]
    fn test_escaping_and_unescaping_trait() {
        let original = "^|*:\\/?\"<>#123^845";

        let escaped = original.escape();
        assert_eq!(escaped, "^^^v^a^c^d^s^q^t^l^r^h123^845");

        let unescaped = &escaped.unescape();
        assert_eq!(unescaped, original);
    }

    #[test]
    fn test_escaping_and_unescaping_trait_in_format() {
        let original = "^|*:\\/?\"<>#123^845";

        let escaped = format!("HELLO WORLD: {}", original.escape());
        assert_eq!(escaped, "HELLO WORLD: ^^^v^a^c^d^s^q^t^l^r^h123^845");

        let unescaped = &escaped.unescape();
        assert_eq!(unescaped, "HELLO WORLD: ^|*:\\/?\"<>#123^845");
    }
}
