//! Utilities for working with 'Codepage strings' from Insim.

/// LFS strings are a sequence of u8 bytes, with an optional trailing \0.
/// The bytes are conventionally compromised of characters from multiple code pages, indicated by a `^` and
/// a following code page identifier character. i.e. `^L` indicates Latin1.
///
/// The common practise is to use the function `to_lossy_string` to convert to a standard Rust
/// String.
use encoding_rs;
use itertools::Itertools;
use once_cell::sync::Lazy;
use std::{borrow::Cow, collections::HashMap, vec::Vec};

use super::{strip_trailing_nul, MARKER};

pub static MAPPING: Lazy<HashMap<u8, &encoding_rs::Encoding>> = Lazy::new(|| {
    let mut m = HashMap::new();

    m.insert(b'L', encoding_rs::WINDOWS_1252);
    m.insert(b'G', encoding_rs::ISO_8859_7);
    m.insert(b'J', encoding_rs::SHIFT_JIS);
    m.insert(b'E', encoding_rs::ISO_8859_2);
    m.insert(b'T', encoding_rs::WINDOWS_1254);
    m.insert(b'B', encoding_rs::ISO_8859_13);
    m.insert(b'H', encoding_rs::GBK);
    m.insert(b'S', encoding_rs::EUC_KR);
    m.insert(b'K', encoding_rs::BIG5);

    m
});

/// Convert from a String, with potential lossy conversion to an Insim Codepage String
pub fn to_lossy_bytes(input: &str) -> Cow<[u8]> {
    let all_ascii = !input.chars().any(|c| c as u32 > 127);

    if all_ascii {
        // all codepages share ascii values
        // therefore if it's all ascii, we can just dump it.
        return input.as_bytes().into();
    }

    let mut output = Vec::new();

    let mut current_encoding = MAPPING.get(&b'L').unwrap();

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
        for (key, val) in MAPPING.iter() {
            if val == current_encoding {
                continue;
            }

            let (cow, _, error) = current_encoding.encode(char_as_bytes);
            if error {
                continue;
            }

            output.push(MARKER);
            output.push(*key);

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
pub fn to_lossy_string(input: &[u8]) -> Cow<str> {
    // empty string
    if input.is_empty() {
        return "".into();
    }

    let input = strip_trailing_nul(input);

    // find the positions in the input for each ^L, ^B...
    let mut indices: Vec<usize> = input
        .iter()
        .tuple_windows()
        .positions(|(elem, next)| *elem == MARKER && MAPPING.contains_key(next))
        .collect();

    if indices.is_empty() {
        // no mappings at all, just encode it all as LATIN1
        let (cow, _encoding, _had_errors) = encoding_rs::WINDOWS_1252.decode(input);
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
        }
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

        if range[0] != MARKER {
            let (cow, _encoding, _had_errors) = encoding_rs::WINDOWS_1252.decode(range);
            result.push_str(&cow);
            continue;
        }

        if let Some(mapping) = MAPPING.get(&range[1]) {
            let (cow, _encoding_used, _had_errors) = mapping.decode(&range[2..]);
            result.push_str(&cow);
        } else {
            // fallback to Latin
            // ensure we include the prefix
            let (cow, _encoding, _had_errors) = encoding_rs::WINDOWS_1252.decode(range);
            result.push_str(&cow);
        }
    }

    result.into()
}
