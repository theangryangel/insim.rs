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
    static MAPPINGS: [(char, &encoding_rs::Encoding); 10] = [
        ('L', encoding_rs::WINDOWS_1252),
        ('C', encoding_rs::WINDOWS_1251),
        ('G', encoding_rs::ISO_8859_7),
        ('J', encoding_rs::SHIFT_JIS),
        ('E', encoding_rs::ISO_8859_2),
        ('T', encoding_rs::WINDOWS_1254),
        ('B', encoding_rs::ISO_8859_13),
        ('H', encoding_rs::GBK),
        ('S', encoding_rs::EUC_KR),
        ('K', encoding_rs::BIG5),
    ];

    pub(crate) fn get(c: char) -> Option<&'static encoding_rs::Encoding> {
        if let Some(index) = MAPPINGS.iter().position(|&r| r.0 == c) {
            Some(MAPPINGS.get(index).unwrap().1)
        } else {
            None
        }
    }

    pub(crate) fn iter() -> impl Iterator<Item = &'static (char, &'static encoding_rs::Encoding)> {
        MAPPINGS.iter()
    }

    pub(crate) fn default() -> &'static encoding_rs::Encoding {
        encoding_rs::WINDOWS_1252
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

    let mut output = Vec::new();
    let mut current_encoding = mappings::default();

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
        for (key, val) in mappings::iter() {
            if *val == current_encoding {
                continue;
            }

            let (cow, _, error) = current_encoding.encode(char_as_bytes);
            if error {
                continue;
            }

            output.push(MARKER as u8);
            output.push(*key as u8);

            output.extend_from_slice(&cow);
            current_encoding = val;

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
        let (cow, _encoding, _had_errors) = mappings::default().decode(input);
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
            let (cow, _encoding, _had_errors) = mappings::default().decode(range);
            result.push_str(&cow);
            continue;
        }

        if let Some(mapping) = mappings::get(range[1] as char) {
            let (cow, _encoding_used, _had_errors) = mapping.decode(&range[2..]);
            result.push_str(&cow);
        } else {
            // fallback to Latin
            // ensure we include the prefix
            let (cow, _encoding, _had_errors) = mappings::default().decode(range);
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
}
