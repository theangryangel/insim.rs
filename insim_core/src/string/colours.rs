//! Utilities for working with colours from Insim.

use std::borrow::Cow;

use super::control::ControlMarker;

/// The effective display colour of an LFS text span.
///
/// Both `^8` (colour and code page reset) and `^9` (colour reset) select
/// [`Self::Default`]. This describes display state, not the original control.
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[repr(u8)]
pub enum ColourCode {
    /// Black.
    Black = 0,
    /// Red.
    Red = 1,
    /// Light Green.
    LightGreen = 2,
    /// Yellow.
    Yellow = 3,
    /// Blue.
    Blue = 4,
    /// Purple.
    Purple = 5,
    /// Light Blue.
    LightBlue = 6,
    /// White.
    White = 7,
    /// The default display colour (dark green).
    #[default]
    Default = 8,
}

/// A character that is not an LFS colour marker.
#[derive(Debug, Clone, Copy, Eq, PartialEq, thiserror::Error)]
#[error("invalid colour marker: {0}")]
pub struct ColourCodeError(pub char);

impl TryFrom<char> for ColourCode {
    type Error = ColourCodeError;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            '0' => Ok(Self::Black),
            '1' => Ok(Self::Red),
            '2' => Ok(Self::LightGreen),
            '3' => Ok(Self::Yellow),
            '4' => Ok(Self::Blue),
            '5' => Ok(Self::Purple),
            '6' => Ok(Self::LightBlue),
            '7' => Ok(Self::White),
            '8' | '9' => Ok(Self::Default),
            _ => Err(ColourCodeError(value)),
        }
    }
}

/// Convert a display colour to its canonical marker, without the `^` prefix.
/// The default colour uses `8`; the original reset instruction is not retained.
impl From<ColourCode> for char {
    fn from(value: ColourCode) -> Self {
        char::from(b'0' + u8::from(value))
    }
}

/// Convert a display colour to its index (`0..=8`).
impl From<ColourCode> for u8 {
    fn from(value: ColourCode) -> Self {
        value as Self
    }
}

// Preserve the distinction between reset controls while parsing markers.
#[derive(Clone, Copy)]
pub(super) enum ColourMarker {
    Colour(ColourCode),
    ResetColourAndCodepage,
    ResetColour,
}

impl TryFrom<char> for ColourMarker {
    type Error = ColourCodeError;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            '8' => Ok(Self::ResetColourAndCodepage),
            '9' => Ok(Self::ResetColour),
            _ => ColourCode::try_from(value).map(Self::Colour),
        }
    }
}

impl ColourMarker {
    fn colour(self) -> ColourCode {
        match self {
            Self::Colour(colour) => colour,
            Self::ResetColourAndCodepage | Self::ResetColour => ColourCode::Default,
        }
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
    /// literal text. Chunks are yielded as `(colour, text)` where `colour` is the effective display colour and `text` is a
    /// slice of the original input. Empty chunks are skipped.
    ///
    /// The yielded spans maybe raw escape sequences (like ^^): You must call unescape
    /// on the text to obtain the final display string.
    ///
    /// Use cases include converting to ansi colours, or html, etc.
    fn colour_spans(&self) -> impl Iterator<Item = (ColourCode, &str)>;
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
        format!("^8{}", self.as_ref())
    }

    fn strip_colours(&self) -> Cow<'_, str> {
        strip(self.as_ref())
    }

    fn colour_spans(&self) -> impl Iterator<Item = (ColourCode, &str)> {
        spans(self.as_ref())
    }
}

/// Split an escaped LFS string into colour chunks.
///
/// This parses colour markers (`^0`..`^9`) while preserving escaped control markers (`^^`) as
/// literal text. Chunks are yielded as `(colour, text)` where `colour` is the effective display colour and `text` is a
/// slice of the original input. Empty chunks are skipped.
///
/// The yielded spans maybe raw escape sequences (like ^^): You must call .unescape()
/// on the text to obtain the final display string.
///
/// Use cases include converting to ansi colours, or html, etc.
pub fn spans(input: &str) -> impl Iterator<Item = (ColourCode, &str)> + '_ {
    let mut iter = input.char_indices().peekable();
    let mut chunk_start = 0;
    let mut current_colour = ColourCode::default();

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

                // ^n (colour code)
                Some(&(next_idx, next)) if let Ok(marker) = ColourMarker::try_from(next) => {
                    let chunk = &input[chunk_start..idx];
                    let yielded_colour = current_colour;

                    // update state for the *next* iteration
                    current_colour = marker.colour();
                    chunk_start = next_idx + next.len_utf8();
                    let _ = iter.next(); // consume the colour digit

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
            && ColourMarker::try_from(*k).is_ok()
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
    fn typed_colour_conversions() {
        let colours = [
            ColourCode::Black,
            ColourCode::Red,
            ColourCode::LightGreen,
            ColourCode::Yellow,
            ColourCode::Blue,
            ColourCode::Purple,
            ColourCode::LightBlue,
            ColourCode::White,
            ColourCode::Default,
        ];
        for (index, colour) in (0_u8..).zip(colours) {
            assert_eq!(u8::from(colour), index);
            assert_eq!(char::from(colour), char::from(b'0' + index));
            assert_eq!(ColourCode::try_from(char::from(colour)), Ok(colour));
        }
        assert_eq!(ColourCode::try_from('9'), Ok(ColourCode::Default));
        for marker in ['^', 'L', 'a', 'é'] {
            assert_eq!(ColourCode::try_from(marker), Err(ColourCodeError(marker)));
        }
    }

    #[test]
    fn reset_controls_select_default_colour() {
        let input = "^1red^9default^2green^8default";
        let expected = vec![
            (ColourCode::Red, "red"),
            (ColourCode::Default, "default"),
            (ColourCode::LightGreen, "green"),
            (ColourCode::Default, "default"),
        ];
        assert_eq!(spans(input).collect::<Vec<_>>(), expected);
        assert_eq!(input.colour_spans().collect::<Vec<_>>(), expected);
        assert_eq!(strip(input), "reddefaultgreendefault");
        assert_eq!(
            spans("^^9").collect::<Vec<_>>(),
            vec![(ColourCode::Default, "^^9")]
        );
        assert_eq!(strip("^^9"), "^^9");
        assert_eq!(crate::string::escaping::escape("^9default"), "^9default");
    }

    #[test]
    fn test_strip_colours_only() {
        assert_eq!(strip("^1^2^3^4^5^6^7^8"), "");
        assert_eq!("^1^2^3^4^5^6^7^8".strip_colours(), "");
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
        assert_eq!(
            spans("abc").collect::<Vec<_>>(),
            vec![(ColourCode::Default, "abc")]
        );
    }

    #[test]
    fn test_colour_spans_with_markers() {
        assert_eq!(
            spans("^1abc ^2efg").collect::<Vec<_>>(),
            vec![(ColourCode::Red, "abc "), (ColourCode::LightGreen, "efg")]
        );
    }

    #[test]
    fn test_colour_spans_escaped_marker_not_colour() {
        assert_eq!(
            spans("^^0").collect::<Vec<_>>(),
            vec![(ColourCode::Default, "^^0")]
        );
    }

    #[test]
    fn test_colour_spans_with_markers_escaped() {
        assert_eq!(
            spans("^1a^^0bc ^2efg").collect::<Vec<_>>(),
            vec![
                (ColourCode::Red, "a^^0bc "),
                (ColourCode::LightGreen, "efg")
            ]
        );
    }

    #[test]
    fn test_colour_spans_greedy_escape_then_colour() {
        assert_eq!(
            spans("^^^1abc").collect::<Vec<_>>(),
            vec![(ColourCode::Default, "^^"), (ColourCode::Red, "abc")]
        );
    }

    #[test]
    fn test_colour_spans_skip_empty_segments() {
        assert_eq!(
            spans("^1^2abc").collect::<Vec<_>>(),
            vec![(ColourCode::LightGreen, "abc")]
        );
    }

    #[test]
    fn test_colour_spans_trailing_colour_marker() {
        assert_eq!(
            spans("abc^1").collect::<Vec<_>>(),
            vec![(ColourCode::Default, "abc")]
        );
    }

    #[test]
    fn test_colourify() {
        assert_eq!(
            "^8The ^0quick ^1brown ^2fox ^3jumps ^4over ^5the ^6lazy ^7dog",
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
