//! Utilities for working with 'Codepage strings' from Insim.

use std::{borrow::Cow, vec::Vec};

/// LFS strings are a sequence of u8 bytes, with an optional trailing \0.
/// The bytes are conventionally compromised of characters from multiple code pages, indicated by a `^` and
/// a following code page identifier character. i.e. `^L` indicates Latin1.
///
/// The common practise is to use the function `to_lossy_string` to convert to a standard Rust
/// String.
use itertools::Itertools;

use super::MARKER;

/// Supported character encoding within LFS
mod mappings {
    pub(super) struct Encoding {
        // escape code for this encoding table
        pub(super) indicator: char,
        // encoding table
        pub(super) table: &'static encoding_rs::Encoding,
        // should we propagate the marker and indicator? i.e. is this ^8 which has dual meaning?
        pub(super) propagate: bool,
    }

    impl Encoding {
        pub const fn new(
            indicator: char,
            table: &'static encoding_rs::Encoding,
            propagate: bool,
        ) -> Self {
            Self {
                indicator,
                table,
                propagate,
            }
        }
    }

    // Maintain the same order as node-insim to try and preserve compatibility with other projects as best as we can https://github.com/simbroadcasts/unicode-to-lfs/blob/main/src/codepages.ts
    static MAPPINGS: [Encoding; 11] = [
        Encoding::new('L', encoding_rs::WINDOWS_1252, false), // Latin-1
        Encoding::new('G', encoding_rs::ISO_8859_7, false),   // Greek
        Encoding::new('C', encoding_rs::WINDOWS_1251, false), // Cyrillic
        Encoding::new('E', encoding_rs::ISO_8859_2, false),   // Central Europe
        Encoding::new('T', encoding_rs::WINDOWS_1254, false), // Turkish
        Encoding::new('B', encoding_rs::ISO_8859_13, false),  // Baltic
        Encoding::new('J', encoding_rs::SHIFT_JIS, false),    // Japanese
        Encoding::new('H', encoding_rs::GBK, false),          // Traditional Chinese
        Encoding::new('S', encoding_rs::EUC_KR, false), // Simplified Chinese, EUC_KR seems to the the closest?
        Encoding::new('K', encoding_rs::BIG5, false), // Should be CP950, BIG5 seems to the closest?
        Encoding::new('8', encoding_rs::WINDOWS_1252, true), // Reset to default
    ];

    pub(super) fn get(c: char) -> Option<&'static Encoding> {
        MAPPINGS.iter().find(|&r| r.indicator == c)
    }

    pub(super) fn iter() -> impl Iterator<Item = &'static Encoding> {
        MAPPINGS.iter()
    }

    pub(super) fn default_encoding() -> &'static Encoding {
        &MAPPINGS[0]
    }
}

/// Convert from a String, with potential lossy conversion to an Insim Codepage String
pub fn to_lossy_bytes(input: &str) -> Cow<[u8]> {
    let all_ascii = !input.chars().any(|c| c as u32 > 127);

    if all_ascii {
        // all codepages share ascii values
        // therefore if it's all ascii, we can just dump it.
        return input.as_bytes().into();
    }

    let mut output = Vec::with_capacity(input.len());
    let mut current_encoding = mappings::default_encoding().table;

    for c in input.chars() {
        // all codepages share ascii values
        if (c as u32) <= 127 {
            output.push(c as u8);
            continue;
        }

        // all utf-8 characters are 3 bytes
        let mut buf = [0; 3];
        let char_as_bytes = c.encode_utf8(&mut buf);

        let (cow, _, error) = current_encoding.encode(char_as_bytes);

        if !error {
            output.extend_from_slice(&cow);
            continue;
        }

        let mut found = false;

        // find an encoding we can use
        for encoding_map in mappings::iter() {
            if encoding_map.table == current_encoding {
                continue;
            }

            let (cow, _, error) = current_encoding.encode(char_as_bytes);
            if error {
                continue;
            }

            output.push(MARKER as u8);
            output.push(encoding_map.indicator as u8);

            output.extend_from_slice(&cow);
            current_encoding = encoding_map.table;

            found = true;
            break;
        }

        if !found {
            // fallback char
            output.push(b'?');
        }
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
        .positions(|(elem, next)| *elem == MARKER as u8 && mappings::get(*next as char).is_some())
        .collect();

    if indices.is_empty() {
        // no mappings at all, just encode it all as LATIN1
        let (cow, _encoding, _had_errors) = mappings::default_encoding().table.decode(input);
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
            result.push_str(&String::from_utf8_lossy(range));
            continue;
        }

        if range[0] != MARKER as u8 {
            let (cow, _encoding, _had_errors) = mappings::default_encoding().table.decode(range);
            result.push_str(&cow);
            continue;
        }

        if let Some(mapping) = mappings::get(range[1] as char) {
            // do we need to propagate the marker?
            if mapping.propagate {
                result.push(MARKER);
                result.push(range[1] as char);
            }

            let (cow, _encoding_used, _had_errors) = mapping.table.decode(&range[2..]);
            result.push_str(&cow);
        } else {
            // fallback to Latin
            // ensure we include the prefix
            let (cow, _encoding, _had_errors) = mappings::default_encoding().table.decode(range);
            result.push_str(&cow);
        }
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

    // sample utf-8 strings from https://www.cl.cam.ac.uk/~mgk25/ucs/examples/quickbrown.txt

    #[test]
    fn test_codepage_to_hungarian() {
        // flood-proof mirror-drilling machine
        let as_bytes = to_lossy_bytes("Árvíztűrő tükörfúrógép");

        assert_eq!(to_lossy_string(&as_bytes), "Árvízt?r? tükörfúrógép",);
    }

    #[test]
    fn test_codepage_to_mixed() {
        // flood-proof mirror-drilling machine
        let as_bytes = to_lossy_bytes("TEST Árvíztűrő tükörfúrógép");

        assert_eq!(to_lossy_string(&as_bytes), "TEST Árvízt?r? tükörfúrógép",);
    }

    #[test]
    fn test_propagate_eight() {
        // flood-proof mirror-drilling machine
        let as_bytes = to_lossy_bytes("^8TEST Árvíztűrő tükörfúrógép");

        assert_eq!(to_lossy_string(&as_bytes), "^8TEST Árvízt?r? tükörfúrógép",);
    }
}
