//! Utilities for working with 'Codepage strings' from Insim.

/// LFS strings are a sequence of u8 bytes, with an optional trailing \0.
/// The bytes are conventionally compromised of characters from multiple code pages, indicated by a `^` and
/// a following code page identifier character. i.e. `^L` indicates Latin1.
///
/// The common practise is to use the function `to_lossy_string` to convert to a standard Rust
/// String.
use encoding_rs;
use if_chain::if_chain;
use itertools::Itertools;
use once_cell::sync::Lazy;
use std::{collections::HashMap, vec::Vec};

use super::{colours::COLOUR_SEQUENCES, strip_trailing_nul, unescape, ESCAPE_SEQUENCES, MARKER};

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
pub fn to_lossy_bytes(input: &str) -> Vec<u8> {
    let mut output = Vec::new();

    let mut current_encoding = MAPPING.get(&b'L').unwrap();

    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        // is the current char a marker? and do we have a follow up character?
        // TODO: replace with a let chain when its stable
        if_chain! {
            if c == MARKER as char;
            if let Some(d) = chars.peek();
            then {
                // is this a colour?
                if COLOUR_SEQUENCES.contains(d) {
                    // just push the colour and move on
                    output.push(MARKER);
                    output.push(chars.next().unwrap() as u8);
                    continue;
                }

                // question: do we want to allow users to pre-escape themselves?
                // probably not.
                // // are we pre-escaped?
                // if ESCAPE_SEQUENCES.iter().find(|i| i.0 as char == *d).is_some() {
                //     // just push and move on
                //     output.push(MARKER);
                //     output.push(chars.next().unwrap() as u8);
                //     continue;
                // }
            }
        }

        // do we have a character that needs escaping?
        if let Some(i) = ESCAPE_SEQUENCES.iter().find(|i| i.1 as char == c) {
            output.push(MARKER);
            output.push(i.0);
            continue;
        }

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

    output
}

/// Convert a InsimString into a native rust String, with potential lossy conversion from codepages
pub fn to_lossy_string(input: &[u8]) -> String {
    // empty string
    if input.is_empty() {
        return "".to_string();
    }

    let input = unescape(strip_trailing_nul(input));

    // find the positions in the input for each ^L, ^B...
    let mut indices: Vec<usize> = input
        .iter()
        .tuple_windows()
        .positions(|(elem, next)| *elem == MARKER && MAPPING.contains_key(next))
        .collect();

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
            result.push_str(&String::from_utf8_lossy(&unescape(range)));
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

    result
}
