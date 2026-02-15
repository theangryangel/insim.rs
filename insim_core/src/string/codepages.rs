//! Utilities for working with 'Codepage strings' from Insim.

use std::{borrow::Cow, vec::Vec};

use smallvec::SmallVec;

/// LFS strings are a sequence of u8 bytes, with an optional trailing \0.
/// The bytes are conventionally compromised of characters from multiple code pages, indicated by a `^` and
/// a following code page identifier character. i.e. `^L` indicates Latin1.
///
/// The common practise is to use the function `to_lossy_string` to convert to a standard Rust
/// String.
use super::control::ControlMarker;

const DEFAULT_CODEPAGE: char = 'L';

trait CodepageMarker {
    /// This is a valid codepage marker/identifier?
    fn is_lfs_codepage(&self) -> bool;
    /// Get the encoding_rs lookup table
    fn as_lfs_codepage(&self) -> Option<&'static encoding_rs::Encoding>;
    /// Should this codepage control character be propagated
    fn propagate_lfs_codepage(self) -> bool;
}

impl CodepageMarker for char {
    fn is_lfs_codepage(&self) -> bool {
        matches!(
            self,
            'L' | 'G' | 'C' | 'E' | 'T' | 'B' | 'J' | 'H' | 'S' | 'K' | '9'
        )
    }

    fn propagate_lfs_codepage(self) -> bool {
        self == '9'
    }

    fn as_lfs_codepage(&self) -> Option<&'static encoding_rs::Encoding> {
        // Some of these are substitutes, based on
        // encoding_rs' generate-encoding-data
        // https://github.com/hsivonen/encoding_rs/blob/acae06412c97df212797bebee9845b9b1c12569b/generate-encoding-data.py

        match self {
            'L' | '9' => Some(encoding_rs::WINDOWS_1252), // Latin-1 CP1252
            'G' => Some(encoding_rs::ISO_8859_7),         // Greek ISO-8859-7
            'C' => Some(encoding_rs::WINDOWS_1251),       // Cyrillic CP1251
            'E' => Some(encoding_rs::ISO_8859_2),         // Central Europe ISO-8859-2
            'T' => Some(encoding_rs::WINDOWS_1254),       // Turkish ISO-8859-9 / CP1254
            'B' => Some(encoding_rs::ISO_8859_13),        // Baltic ISO-8859-13 / Latin-7
            'J' => Some(encoding_rs::SHIFT_JIS),          // Japanese SHIFT-JIS
            'H' => Some(encoding_rs::GBK),                // Traditional Chinese CP936
            'S' => Some(encoding_rs::EUC_KR),             // Simplified Chinese CP949
            'K' => Some(encoding_rs::BIG5),               // Korean CP950
            _ => None,                                    // Not a codepage
        }
    }
}

impl CodepageMarker for u8 {
    fn is_lfs_codepage(&self) -> bool {
        (*self as char).is_lfs_codepage()
    }

    fn propagate_lfs_codepage(self) -> bool {
        (self as char).propagate_lfs_codepage()
    }

    fn as_lfs_codepage(&self) -> Option<&'static encoding_rs::Encoding> {
        (*self as char).as_lfs_codepage()
    }
}

/// Convert from a String, with potential lossy conversion to an Insim Codepage String
/// Assumes you will escape any characters ahead of time, it will do not this for you.
/// See <https://github.com/theangryangel/insim.rs/issues/92> for further details.
pub fn to_lossy_bytes(input: &'_ str) -> Cow<'_, [u8]> {
    if input.is_ascii() {
        // all codepages share ascii values
        // therefore if it's all ascii, we can just dump it.
        return input.as_bytes().into();
    }

    let mut output = Vec::with_capacity(input.len());
    let mut current_control = DEFAULT_CODEPAGE;
    let mut current_encoding = current_control
        .as_lfs_codepage()
        .unwrap_or_else(|| unreachable!());
    let mut encoder = current_encoding.new_encoder();

    let mut src_offset = 0;
    // worst case output size: multi-byte encodings can expand, but 2x /probably/ is safe enough
    // without being too wasteful?
    let mut dst = vec![0u8; input.len() * 2];

    while src_offset < input.len() {
        let (res, src_read, dst_written) = encoder.encode_from_utf8_without_replacement(
            &input[src_offset..],
            &mut dst,
            true, // we always have the full input
        );

        // push whatever was successfully encoded
        output.extend_from_slice(&dst[..dst_written]);
        src_offset += src_read;

        match res {
            encoding_rs::EncoderResult::InputEmpty => break,
            encoding_rs::EncoderResult::Unmappable(c) => {
                let mut buf = [0u8; 4];
                let char_as_bytes = c.encode_utf8(&mut buf);

                let Some(&candidate_control) = super::codepages_lut::lookup(c)
                    .iter()
                    .find(|&&candidate| candidate != current_control)
                else {
                    // We found nothing, post the fallback character
                    output.push(b'?');
                    continue;
                };

                let candidate_encoding = candidate_control
                    .as_lfs_codepage()
                    .unwrap_or_else(|| unreachable!());
                let (cow, _, error) = candidate_encoding.encode(char_as_bytes);

                // Lookup table guarantees this, but be defensive in case of drift.
                if error {
                    output.push(b'?');
                    continue;
                }

                // this one matched, push the control character and codepage
                // control character
                output.push(u8::lfs_control_char());
                output.push(candidate_control as u8);
                // then push the new character
                output.extend_from_slice(&cow);
                // switch the encoder to the new codepage for the remainder
                current_control = candidate_control;
                current_encoding = candidate_encoding;
                encoder = current_encoding.new_encoder();
            },
            encoding_rs::EncoderResult::OutputFull => {
                // dst wasn't big enough resize and retry from the same offset
                dst.resize(dst.len() * 2, 0);
            },
        }
    }

    output.into()
}

/// Convert a InsimString into a native rust String, with potential lossy conversion from codepages
/// Assumes any \0 characters have been stripped ahead of time
///
/// This decodes codepage markers only. It does not unescape LFS escape sequences.
pub fn to_lossy_string(input: &'_ [u8]) -> Cow<'_, str> {
    // empty string
    if input.is_empty() {
        return "".into();
    }

    // allowing unwrap because if this panics we're screwed
    let default_lfs_codepage = DEFAULT_CODEPAGE
        .as_lfs_codepage()
        .unwrap_or_else(|| unreachable!());

    // fastest possible path - we have no *potential* control characters at all
    if !input.iter().any(|b| b.is_lfs_control_char()) {
        if input.is_ascii() {
            return Cow::Borrowed(
                str::from_utf8(input)
                    .expect("we already checked if this was only ascii characters"),
            );
        }

        let (cow, _, _) = default_lfs_codepage.decode(input);
        return cow;
    }

    // slowest path
    // find the positions in the input for each ^L, ^B...
    // XXX: Using SmallVec here to avoid an allocation if possible
    let mut indices: SmallVec<[usize; 8]> = SmallVec::new();
    let mut iter = input.iter().enumerate().peekable();

    while let Some((i, &cur)) = iter.next() {
        // we only care if the current char is a control char
        if !cur.is_lfs_control_char() {
            continue;
        }

        if let Some((_, next)) = iter.peek() {
            if next.is_lfs_control_char() {
                // greedy handling for escaped control markers (^^)
                // we consume the next char so we don't process it again
                let _ = iter.next();
            } else if next.is_lfs_codepage() {
                // found a valid codepage marker (^L, ^B, etc)
                indices.push(i);
                // consume the next char as part of this marker
                let _ = iter.next();
            }
        }
    }

    // allowing unwrap because if this panics we're screwed
    let default_lfs_codepage = DEFAULT_CODEPAGE
        .as_lfs_codepage()
        .unwrap_or_else(|| unreachable!());

    if indices.is_empty() {
        // no mappings at all, just encode it all as the default
        let (cow, _encoding, _had_errors) = default_lfs_codepage.decode(input);
        return cow;
    }

    // make sure we've got at least something in the indices
    if indices.first() != Some(&0) {
        indices.insert(0, 0);
    }

    // make sure we've got the last item in here as well
    match indices.last() {
        Some(last) => {
            if *last != input.len() {
                indices.push(input.len());
            }
        },
        None => indices.push(input.len()),
    };

    // This pre-allocation is the best guess we can make here
    let mut result = String::with_capacity(input.len());

    for pair in indices.windows(2) {
        let range = &input[pair[0]..pair[1]];

        if range.len() < 2 {
            let (cow, _encoding, _had_errors) = default_lfs_codepage.decode(range);
            result.push_str(&cow);
            continue;
        }

        match (range[0].is_lfs_control_char(), range[1].as_lfs_codepage()) {
            (false, _) | (true, None) => {
                // No control character
                // OR
                // Has a control character, but next character is not a codepage
                // THEN
                // fallback to default codepage and ensure we include the prefix
                let (cow, _encoding, _had_errors) = default_lfs_codepage.decode(range);
                result.push_str(&cow);
            },
            (true, Some(mapping)) => {
                // Has a control character and next character is a codepage

                // do we need to propagate the codepage because it has dual meaning?
                // i.e. ^9
                if range[1].propagate_lfs_codepage() {
                    result.push(char::lfs_control_char());
                    result.push(range[1] as char);
                }

                // encode everything except the markers
                let (cow, _encoding_used, _had_errors) = mapping.decode(&range[2..]);
                result.push_str(&cow);
            },
        };
    }

    result.into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_codepage_hello_world() {
        let output = to_lossy_bytes("Hello");

        assert_eq!(output, "Hello".as_bytes(),);
    }

    #[test]
    fn test_keep_ascii() {
        let raw = " ! \" # $ % & ' ( ) * + , - . / 0 1 2 3 4 5 6 7 8 9 : ; < = > ? @ A B C D E F G H I J K L M N O P Q R S T U V W X Y Z [ \\ ] ^ _ ` a b c d e f g h i j k l m n o p q r s t u v w x y z { | } ~";
        let output = to_lossy_bytes(raw);

        assert_eq!(output, raw.as_bytes(),);
    }

    #[test]
    fn test_all_codepages() {
        let raw = [
            // (expected codepage, input utf-8)
            // Each item should include some ascii characters AND codepage required items to ensure
            // that we dont start adding extra Latin-1 encodings when not necessary.
            // This is because all codepages share ascii
            ('E', "ěš 9 "),
            ('C', "шю 10 "),
            ('L', "ýþ 12 "),
            ('G', "ώλ 13 "),
            ('T', "ış 14 "),
            ('B', "ūņ 15 "),
            ('J', "ﾏ美 16"),
        ];

        let input = raw.iter().map(|e| e.1).collect::<String>();

        let generated_output = to_lossy_bytes(&input);

        let mut expected_output: Vec<u8> = Vec::new();
        raw.iter().for_each(|x| {
            // Not using fold to avoid the copying of the accumulator
            let cp = x.0.as_lfs_codepage().unwrap_or_else(|| unreachable!());
            let (cow, _, error) = cp.encode(x.1);
            assert!(!error);
            expected_output.push(b'^');
            expected_output.push(x.0 as u8);
            expected_output.extend_from_slice(&cow);
        });

        // did the generated output match what we think it should be?
        assert_eq!(generated_output, expected_output);

        // when we convert both the generated and expected output back, do they match the original
        // input?
        assert_eq!(input, to_lossy_string(&generated_output));
        assert_eq!(input, to_lossy_string(&expected_output));
    }

    // sample utf-8 strings from https://www.cl.cam.ac.uk/~mgk25/ucs/examples/quickbrown.txt

    #[test]
    fn test_codepage_to_hungarian() {
        // flood-proof mirror-drilling machine
        let as_bytes = to_lossy_bytes("Árvíztűrő tükörfúrógép");

        assert_eq!(to_lossy_string(&as_bytes), "Árvíztűrő tükörfúrógép",);
    }

    #[test]
    fn test_codepage_to_mixed() {
        // flood-proof mirror-drilling machine
        let as_bytes = to_lossy_bytes("TEST Árvíztűrő tükörfúrógép");

        assert_eq!(to_lossy_string(&as_bytes), "TEST Árvíztűrő tükörfúrógép",);
    }

    #[test]
    fn test_propagate_nine() {
        // flood-proof mirror-drilling machine
        let as_bytes = to_lossy_bytes("^9TEST");

        assert_eq!(to_lossy_string(&as_bytes), "^9TEST",);
    }

    #[test]
    fn test_retain_colours() {
        let raw = "^1abc ^2efg";
        let as_bytes = to_lossy_bytes(&raw);

        assert_eq!(as_bytes, raw.as_bytes());
    }

    #[test]
    fn test_retain_escaping() {
        let raw = "^^";
        let as_bytes = to_lossy_bytes(&raw);
        assert_eq!(as_bytes, raw.as_bytes());
        assert_eq!(raw, to_lossy_string(&as_bytes));
    }

    #[test]
    fn test_escaped_codepage_does_not_convert() {
        for codepage in ['L', 'G', 'C', 'E', 'T', 'B', 'J', 'H', 'S', 'K', '9'] {
            let raw = format!("^^{}1", codepage);
            let as_string = to_lossy_string(raw.as_bytes());
            assert_eq!(as_string, raw);
        }
    }

    #[test]
    fn test_does_not_escape() {
        let raw = "| test | * : \\ / ? \" < > # ^";
        assert_eq!(to_lossy_bytes(raw), raw.as_bytes());
    }
}
