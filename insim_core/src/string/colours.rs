//! Utilities for working with colours from Insim.

use std::borrow::Cow;

use super::control::ControlMarker;

/// Trait to help identify colour markers/identifiers
/// Left public to allow users to implement their own variation on colour stripping or replacement.
/// i.e. ASCII or HTML.
pub(super) trait ColourMarker {
    /// Is this a supported colour control character within LFS
    fn is_lfs_colour(&self) -> bool;
}

impl ColourMarker for char {
    fn is_lfs_colour(&self) -> bool {
        matches!(
            self,
            '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9'
        )
    }
}

impl ColourMarker for u8 {
    fn is_lfs_colour(&self) -> bool {
        (*self as char).is_lfs_colour()
    }
}

/// Trait to help build coloured strings. API is heavily inspired by colored-rs/colored.
pub trait Colour {
    /// Make this black
    fn black(self) -> String;
    /// Make this red
    fn red(self) -> String;
    /// Make this light green
    fn light_green(self) -> String;
    /// Make this yellow
    fn yellow(self) -> String;
    /// Make this blue
    fn blue(self) -> String;
    /// Make this purple
    fn purple(self) -> String;
    /// Make this light blue
    fn light_blue(self) -> String;
    /// Make this white
    fn white(self) -> String;
    /// Make this dark green (default colour)
    fn dark_green(self) -> String;

    /// Strip colours from a string.
    ///
    /// If you also need to unescape, strip colours first while marker intent is still preserved.
    fn strip_colours(&self) -> Cow<'_, str>;

    /// Split an escaped LFS string into colour spans/chunks.
    ///
    /// This parses colour markers (`^0`..`^9`) while preserving escaped control markers (`^^`) as
    /// literal text. Chunks are yielded as `(colour, text)` where `colour` is `0..=9` and `text` is a
    /// slice of the original input. Empty chunks are skipped.
    ///
    /// The yielded spans maybe raw escape sequences (like ^^): You must call unescape
    /// on the text to obtain the final display string.
    ///
    /// Use cases include converting to ansi colours, or html, etc.
    fn colour_spans(&self) -> impl Iterator<Item = (u8, &str)>;
}

impl<T: AsRef<str>> Colour for T {
    fn black(self) -> String {
        format!("^0{}", self.as_ref())
    }

    fn red(self) -> String {
        format!("^1{}", self.as_ref())
    }

    fn light_green(self) -> String {
        format!("^2{}", self.as_ref())
    }

    fn yellow(self) -> String {
        format!("^3{}", self.as_ref())
    }

    fn blue(self) -> String {
        format!("^4{}", self.as_ref())
    }

    fn purple(self) -> String {
        format!("^5{}", self.as_ref())
    }

    fn light_blue(self) -> String {
        format!("^6{}", self.as_ref())
    }

    fn white(self) -> String {
        format!("^7{}", self.as_ref())
    }

    fn dark_green(self) -> String {
        format!("^9{}", self.as_ref())
    }

    fn strip_colours(&self) -> Cow<'_, str> {
        strip(self.as_ref())
    }

    fn colour_spans(&self) -> impl Iterator<Item = (u8, &str)> {
        spans(self.as_ref())
    }
}

/// Split an escaped LFS string into colour chunks.
///
/// This parses colour markers (`^0`..`^9`) while preserving escaped control markers (`^^`) as
/// literal text. Chunks are yielded as `(colour, text)` where `colour` is `0..=9` and `text` is a
/// slice of the original input. Empty chunks are skipped.
///
/// The yielded spans maybe raw escape sequences (like ^^): You must call .unescape()
/// on the text to obtain the final display string.
///
/// Use cases include converting to ansi colours, or html, etc.
pub fn spans(input: &str) -> impl Iterator<Item = (u8, &str)> + '_ {
    let mut iter = input.char_indices().peekable();
    let mut chunk_start = 0;
    let mut current_colour = 9;

    std::iter::from_fn(move || {
        while let Some((idx, ch)) = iter.next() {
            if !ch.is_lfs_control_char() {
                continue;
            }

            match iter.peek() {
                // ^^ (escaped control char)
                Some(&(_, next)) if next.is_lfs_control_char() => {
                    // we consume the second caret so it isn't processed as a start.
                    // we do not update chunk_start, effectively keeping "^^" in the text.
                    let _ = iter.next();
                },

                // ^n (color code)
                Some(&(next_idx, next)) if next.is_lfs_colour() => {
                    let chunk = &input[chunk_start..idx];
                    let yielded_colour = current_colour;

                    // update state for the *next* iteration
                    current_colour = next.to_digit(10).unwrap_or(0) as u8;
                    chunk_start = next_idx + next.len_utf8();
                    let _ = iter.next(); // consume the color digit

                    // only yield if there is actual text (skips empty chunks like ^1^2)
                    if !chunk.is_empty() {
                        return Some((yielded_colour, chunk));
                    }
                },

                // everything else
                _ => {},
            }
        }

        // yield any remaining text
        if chunk_start < input.len() {
            let chunk = &input[chunk_start..];
            chunk_start = input.len();
            return Some((current_colour, chunk));
        }

        // jobs done
        None
    })
}

/// Strip LFS colours
///
/// If you also need to unescape, call this before unescaping so escaped markers (`^^`) are still
/// distinguishable from real colour markers.
/// Prefer the [`Colour::strip_colours`] trait function
pub fn strip(input: &'_ str) -> Cow<'_, str> {
    if !input.chars().any(|c| c.is_lfs_control_char()) {
        return Cow::Borrowed(input);
    }

    let mut iter = input.chars().peekable();
    let mut output = String::with_capacity(input.len());

    while let Some(i) = iter.next() {
        let j = iter.peek();

        if i.is_lfs_control_char()
            && let Some(k) = j
            && k.is_lfs_control_char()
        {
            // Special case, ignore escaped markers (AKA ^^)
            // If we don't do this now, and just fall through the next check, something like ^^1
            // wont be handled correctly!

            output.push(i);
            output.push(*k);
            let _ = iter.next();
            continue;
        }

        if i.is_lfs_control_char()
            && let Some(k) = j
            && k.is_lfs_colour()
        {
            let _ = iter.next();
            continue;
        }

        output.push(i);
    }

    output.into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_colours_only() {
        assert_eq!(strip("^1^2^3^4^5^6^7^8^9"), "");
        assert_eq!("^1^2^3^4^5^6^7^8^9".strip_colours(), "");
    }

    #[test]
    fn test_strip_colours() {
        assert_eq!(strip("^1234^56789"), "2346789");
        assert_eq!("^1234^56789".strip_colours(), "2346789");
    }

    #[test]
    fn test_strip_colours_escaped() {
        assert_eq!(strip("^^1234^56789"), "^^12346789");
    }

    #[test]
    fn test_colour_spans_default() {
        assert_eq!(spans("abc").collect::<Vec<_>>(), vec![(9, "abc")]);
    }

    #[test]
    fn test_colour_spans_with_markers() {
        assert_eq!(
            spans("^1abc ^2efg").collect::<Vec<_>>(),
            vec![(1, "abc "), (2, "efg")]
        );
    }

    #[test]
    fn test_colour_spans_escaped_marker_not_colour() {
        assert_eq!(spans("^^0").collect::<Vec<_>>(), vec![(9, "^^0")]);
    }

    #[test]
    fn test_colour_spans_with_markers_escaped() {
        assert_eq!(
            spans("^1a^^0bc ^2efg").collect::<Vec<_>>(),
            vec![(1, "a^^0bc "), (2, "efg")]
        );
    }

    #[test]
    fn test_colour_spans_greedy_escape_then_colour() {
        assert_eq!(
            spans("^^^1abc").collect::<Vec<_>>(),
            vec![(9, "^^"), (1, "abc")]
        );
    }

    #[test]
    fn test_colour_spans_skip_empty_segments() {
        assert_eq!(spans("^1^2abc").collect::<Vec<_>>(), vec![(2, "abc")]);
    }

    #[test]
    fn test_colour_spans_trailing_colour_marker() {
        assert_eq!(spans("abc^1").collect::<Vec<_>>(), vec![(9, "abc")]);
    }

    #[test]
    fn test_colourify() {
        assert_eq!(
            "^9The ^0quick ^1brown ^2fox ^3jumps ^4over ^5the ^6lazy ^7dog",
            format!(
                "{} {} {} {} {} {} {} {} {}",
                "The".dark_green(),
                "quick".black(),
                "brown".red(),
                "fox".light_green(),
                "jumps".yellow(),
                "over".blue(),
                "the".purple(),
                "lazy".light_blue(),
                "dog".white()
            )
        );
    }

    #[test]
    fn test_colourify_string() {
        assert_eq!("^4Test", String::from("Test").blue());
    }

    #[test]
    fn test_colourify_str() {
        assert_eq!("^4Test", "Test".blue());
    }
}
