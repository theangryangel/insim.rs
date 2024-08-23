//! Utilities for working with 'Codepage strings' from Insim.

use std::{borrow::Cow, vec::Vec};

/// LFS strings are a sequence of u8 bytes, with an optional trailing \0.
/// The bytes are conventionally compromised of characters from multiple code pages, indicated by a `^` and
/// a following code page identifier character. i.e. `^L` indicates Latin1.
///
/// The common practise is to use the function `to_lossy_string` to convert to a standard Rust
/// String.
use itertools::Itertools;

use super::control::ControlCharacter;

const DEFAULT_CODEPAGE: char = 'L';
// 8 is left off this by design to prevent double checking LATIN1
const VALID_CODEPAGES_FOR_ENCODING: [char; 10] = ['L', 'G', 'C', 'E', 'T', 'B', 'J', 'H', 'S', 'K'];

trait Codepage {
    /// This is a valid codepage marker/identifier?
    fn is_lfs_codepage(&self) -> bool;
    /// Get the encoding_rs lookup table
    fn as_lfs_codepage(&self) -> Option<&'static encoding_rs::Encoding>;
    /// Should this codepage control character be propagated
    fn propagate_lfs_codepage(self) -> bool;
}

impl Codepage for char {
    fn is_lfs_codepage(&self) -> bool {
        matches!(
            self,
            'L' | 'G' | 'C' | 'E' | 'T' | 'B' | 'J' | 'H' | 'S' | 'K' | '8'
        )
    }

    fn propagate_lfs_codepage(self) -> bool {
        self == '8'
    }

    fn as_lfs_codepage(&self) -> Option<&'static encoding_rs::Encoding> {
        // Some of these are substitutes, based on
        // encoding_rs' generate-encoding-data
        // https://github.com/hsivonen/encoding_rs/blob/acae06412c97df212797bebee9845b9b1c12569b/generate-encoding-data.py

        match self {
            'L' | '8' => Some(encoding_rs::WINDOWS_1252), // Latin-1 CP1252
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

impl Codepage for u8 {
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
pub fn to_lossy_bytes(input: &str) -> Cow<[u8]> {
    if input.chars().all(|c| c.is_ascii()) {
        // all codepages share ascii values
        // therefore if it's all ascii, we can just dump it.
        return input.as_bytes().into();
    }

    let mut output = Vec::with_capacity(input.len());
    let mut current_control = DEFAULT_CODEPAGE;
    let mut current_encoding = current_control
        .as_lfs_codepage()
        .unwrap_or_else(|| unreachable!());
    // a succulent buffer for reuse, we'll zero it before each use.
    // all utf-8 characters are no longer than 4 bytes.
    let mut buf = [0; 4];

    'outer: for c in input.chars() {
        // all codepages share ascii values
        if c.is_ascii() {
            output.push(c as u8);
            continue;
        }

        buf.fill(0);
        let char_as_bytes = c.encode_utf8(&mut buf);

        // allowing unwrap because we should never get to a position where we cannot have one
        let (cow, _, error) = current_encoding.encode(char_as_bytes);

        if !error {
            output.extend_from_slice(&cow);
            continue;
        }

        // try to find an encoding we can use
        for candidate_control in VALID_CODEPAGES_FOR_ENCODING {
            // we've already checked the current codepage and failed, don't check again, try the
            // next codepage
            if candidate_control == current_control {
                continue;
            }

            let candidate_encoding = candidate_control
                .as_lfs_codepage()
                .unwrap_or_else(|| unreachable!());

            // try to encode the current character
            let (cow, _, error) = candidate_encoding.encode(char_as_bytes);
            if error {
                // this codepage doesnt match, try the next one
                continue;
            }

            // this one matched, push the control character and codepage control character
            output.push(u8::lfs_control_char());
            output.push(candidate_control as u8);

            // then push the new character
            output.extend_from_slice(&cow);
            // make sure for the next loop that we're going to try the same codepage again
            current_encoding = candidate_encoding;
            current_control = candidate_control;

            // continue the outer loop
            continue 'outer;
        }

        // We found nothing, post the fallback character
        // fallback char
        output.push(b'?');
    }

    output.into()
}

/// Convert a InsimString into a native rust String, with potential lossy conversion from codepages
/// Assumes any \0 characters have been stripped ahead of time
pub fn to_lossy_string(input: &[u8]) -> Cow<str> {
    // empty string
    if input.is_empty() {
        return "".into();
    }

    // find the positions in the input for each ^L, ^B...
    let mut indices: Vec<usize> = input
        .iter()
        .tuple_windows()
        .positions(|(elem, next)| elem.is_lfs_control_char() && next.is_lfs_codepage())
        .collect();

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
                // i.e. ^8
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
    fn test_propagate_eight() {
        // flood-proof mirror-drilling machine
        let as_bytes = to_lossy_bytes("^8TEST");

        assert_eq!(to_lossy_string(&as_bytes), "^8TEST",);
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
    }

    #[test]
    fn test_does_not_escape() {
        let raw = "| test | * : \\ / ? \" < > # ^";
        assert_eq!(to_lossy_bytes(raw), raw.as_bytes());
    }
}
