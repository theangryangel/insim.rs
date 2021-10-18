use encoding_rs;
use itertools::Itertools;
use std::vec::Vec;

mod impl_deku;
mod impl_std;

const CODEPAGES: &[u8] = &[b'L', b'G', b'J', b'E', b'T', b'B', b'H', b'S', b'K'];
const CODEPAGE_MARKER: u8 = b'^';

#[derive(PartialEq, Default)]
pub struct InsimString {
    inner: String,
}

impl InsimString {
    pub const fn new() -> InsimString {
        InsimString {
            inner: String::new(),
        }
    }

    pub fn drop_colours() -> InsimString {
        unimplemented!()
    }

    pub fn ascii_colours() -> InsimString {
        unimplemented!()
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.len() == 0
    }

    pub fn clear(&mut self) {
        self.inner.clear()
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.inner.into_bytes()
    }

    pub fn from_string(value: String) -> InsimString {
        InsimString { inner: value }
    }

    pub fn into_insim(&self, size: usize) -> Vec<u8> {
        // TODO we can do this without allocating a buffer, etc.
        // TODO we need to be able to convert from utf-8 back to LFS' format
        let mut buf = self.inner.as_bytes().to_vec();
        if buf.len() < size {
            buf.reserve(size - buf.len());
            for _i in 0..(size - buf.len()) {
                buf.push(0);
            }
        }
        buf[0..size].to_vec()
    }

    pub fn from_insim(value: Vec<u8>) -> InsimString {
        // TODO Split this up

        // remove trailing \0
        let input = if let Some(rpos) = value.iter().rposition(|x| *x != 0) {
            &value[..=rpos]
        } else {
            &value
        };

        // empty string
        if input.is_empty() {
            return InsimString::new();
        }

        // find the positions in the input for each ^L, ^B...
        let mut indices: Vec<usize> = input
            .iter()
            .tuple_windows()
            .positions(|(elem, next)| *elem == CODEPAGE_MARKER && CODEPAGES.contains(next))
            .collect();

        // make sure we've got at least something in the indices
        if indices.get(0) != Some(&0) {
            indices.insert(0, 0);
        }

        // make sure we've got the last item in here as well
        if *indices.last().unwrap() != input.len() {
            indices.push(input.len());
        }

        let mut result = String::new();

        for pair in indices.windows(2) {
            let range = &input[pair[0]..pair[1]];

            // TODO: unescape

            if range.len() < 2 {
                result.push_str(&String::from_utf8(range.to_vec()).unwrap());
                continue;
            }

            // TODO: Turn this into a static/lazy hashmap lookup or something
            let decoded = match range[0..2] {
                [b'^', b'G'] => {
                    let (cow, _encoding_used, _had_errors) =
                        encoding_rs::ISO_8859_7.decode(&range[2..]);
                    cow.to_string()
                }

                [b'^', b'C'] => {
                    let (cow, _encoding_used, _had_errors) =
                        encoding_rs::ISO_8859_5.decode(&range[2..]);
                    cow.to_string()
                }

                [b'^', b'J'] => {
                    let (cow, _encoding_used, _had_errors) =
                        encoding_rs::SHIFT_JIS.decode(&range[2..]);
                    cow.to_string()
                }

                [b'^', b'E'] => {
                    let (cow, _encoding_used, _had_errors) =
                        encoding_rs::ISO_8859_2.decode(&range[2..]);
                    cow.to_string()
                }

                [b'^', b'T'] => {
                    let (cow, _encoding_used, _had_errors) =
                        encoding_rs::WINDOWS_1254.decode(&range[2..]);
                    cow.to_string()
                }

                [b'^', b'B'] => {
                    let (cow, _encoding_used, _had_errors) =
                        encoding_rs::ISO_8859_4.decode(&range[2..]);
                    cow.to_string()
                }

                [b'^', b'H'] => {
                    let (cow, _encoding_used, _had_errors) = encoding_rs::BIG5.decode(&range[2..]);
                    cow.to_string()
                }

                [b'^', b'S'] => {
                    let (cow, _encoding_used, _had_errors) = encoding_rs::GBK.decode(&range[2..]);
                    cow.to_string()
                }

                [b'^', b'K'] => {
                    let (cow, _encoding_used, _had_errors) =
                        encoding_rs::ISO_8859_7.decode(&range[2..]);
                    cow.to_string()
                }

                [b'^', b'L'] => {
                    let (cow, _encoding, _had_errors) =
                        encoding_rs::WINDOWS_1252.decode(&range[2..]);
                    cow.to_string()
                    //String::from_utf8(range[2..].to_vec()).unwrap()
                }

                // Latin fallback, no prefix
                _ => {
                    let (cow, _encoding, _had_errors) = encoding_rs::WINDOWS_1252.decode(range);
                    cow.to_string()
                }
            };

            result.push_str(&decoded);
        }

        // TODO: We should implement some error handling in here

        InsimString { inner: result }
    }
}
