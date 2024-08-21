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

const DEFAULT_LFS_CODEPAGE_MARKER: char = 'L';
// 8 is left off this by design to prevent double checking LATIN1
const ALL_LFS_CODEPAGE_MARKERS_FOR_ENCODING: [char; 10] =
    ['L', 'G', 'C', 'E', 'T', 'B', 'J', 'H', 'S', 'K'];

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
        match self {
            'L' | '8' => Some(encoding_rs::WINDOWS_1252),
            'G' => Some(encoding_rs::ISO_8859_7),
            'C' => Some(encoding_rs::WINDOWS_1251),
            'E' => Some(encoding_rs::ISO_8859_2),
            'T' => Some(encoding_rs::WINDOWS_1254),
            'B' => Some(encoding_rs::ISO_8859_13),
            'J' => Some(encoding_rs::SHIFT_JIS),
            'H' => Some(encoding_rs::GBK),
            'S' => Some(encoding_rs::EUC_KR),
            'K' => Some(encoding_rs::BIG5),
            _ => None,
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
pub fn to_lossy_bytes(input: &str) -> Cow<[u8]> {
    if input.chars().all(|c| c.is_ascii()) {
        // all codepages share ascii values
        // therefore if it's all ascii, we can just dump it.
        return input.as_bytes().into();
    }

    let mut output = Vec::with_capacity(input.len());
    let mut current_encoding = DEFAULT_LFS_CODEPAGE_MARKER;

    // a succulent buffer for reuse, we'll zero it before each use.
    // all utf-8 characters are 3 bytes.
    let mut buf = [0; 3];

    'outer: for c in input.chars() {
        // all codepages share ascii values
        if c.is_ascii() {
            output.push(c as u8);
            continue;
        }

        // zero the buffer
        buf.fill(0);
        let char_as_bytes = c.encode_utf8(&mut buf);

        // allowing unwrap because we should never get to a position where we cannot have one
        let (cow, _, error) = current_encoding
            .as_lfs_codepage()
            .unwrap()
            .encode(char_as_bytes);

        if !error {
            output.extend_from_slice(&cow);
            continue;
        }

        // try to find an encoding we can use
        for codepage_marker in ALL_LFS_CODEPAGE_MARKERS_FOR_ENCODING {
            // we've already checked the current codepage and failed, don't check again, try the
            // next codepage
            if codepage_marker == current_encoding {
                continue;
            }

            // try to encode the current character
            let (cow, _, error) = current_encoding
                .as_lfs_codepage()
                .unwrap()
                .encode(char_as_bytes);
            if error {
                // this codepage doesnt match, try the next one
                continue;
            }

            // this one matched, push the control character and codepage control character
            output.push(u8::lfs_control_char());
            output.push(codepage_marker as u8);

            // then push the new character
            output.extend_from_slice(&cow);
            // make sure for the next loop that we're going to try the same codepage again
            current_encoding = codepage_marker;

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
    let default_lfs_codepage = DEFAULT_LFS_CODEPAGE_MARKER.as_lfs_codepage().unwrap();

    if indices.is_empty() {
        // no mappings at all, just encode it all as the default
        // allow unwrap because we should always have something here
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

        match (
            (range.len() < 2),
            range[0].is_lfs_control_char(),
            range[1].as_lfs_codepage(),
        ) {
            (true, _, _) | (false, false, _) | (false, true, None) => {
                // No marker, encode everything
                // OR
                // Has a marker, but no mapping
                // fallback to default codepage and ensure we include the prefix
                let (cow, _encoding, _had_errors) = default_lfs_codepage.decode(range);
                result.push_str(&cow);
            },
            (false, true, Some(mapping)) => {
                // Has a marker and a mapping

                // do we need to propagate the marker?
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
