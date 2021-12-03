use encoding_rs;
use itertools::Itertools;
use std::vec::Vec;

mod impl_deku;
mod impl_serde;
mod impl_std;

const CODEPAGES: &[u8] = &[b'L', b'G', b'J', b'E', b'T', b'B', b'H', b'S', b'K'];
const CODEPAGE_MARKER: u8 = b'^';

#[derive(PartialEq, Default, Debug)]
pub struct InsimString {
    inner: Vec<u8>,
}

impl InsimString {
    pub const fn new() -> InsimString {
        InsimString { inner: Vec::new() }
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Clear the inner vec
    pub fn clear(&mut self) {
        self.inner.clear()
    }

    /// Strip any trailing \0 characters from the inner vec.
    pub fn strip_trailing_nul(&mut self) {
        // remove trailing \0
        let value = &self.inner;

        self.inner = if let Some(rpos) = value.iter().rposition(|x| *x != 0) {
            &value[..=rpos]
        } else {
            &value
        }
        .to_vec();
    }

    pub fn from_bytes(input: Vec<u8>) -> Self {
        let mut s = InsimString { inner: input };
        s.strip_trailing_nul();
        s
    }

    pub fn into_bytes(&self) -> &Vec<u8> {
        &self.inner
    }

    /// Convert from a String using the default LFS encoding ("Latin1")
    pub fn from_string(value: String) -> InsimString {
        let (output, _encoding, _had_errors) = encoding_rs::WINDOWS_1252.encode(&value);
        InsimString {
            inner: output.to_vec(),
        }
    }

    /// Convert a InsimString into a native rust String, with potential lossy conversion from codepages
    pub fn to_lossy_string(&self) -> String {
        // TODO Split this up
        let input = &self.inner;

        // empty string
        if input.is_empty() {
            return "".to_string();
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

            // FIXME: unescape

            if range.len() < 2 {
                result.push_str(&String::from_utf8(range.to_vec()).unwrap());
                continue;
            }

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

        result
    }

    /// Builder function to convert a rust native string into Latin1 and push onto inner, prefixed with formatting commands.
    pub fn l(mut self, value: String) -> Self {
        self.inner.extend_from_slice(&[b'^', b'L']);
        let (output, _encoding, _had_errors) = encoding_rs::WINDOWS_1252.encode(&value);
        self.inner.extend_from_slice(&output);
        self
    }

    /// Builder function to convert a rust native string into Greek and push onto inner, prefixed with formatting commands.
    pub fn g(mut self, value: String) -> Self {
        self.inner.extend_from_slice(&[b'^', b'G']);
        let (output, _encoding, _had_errors) = encoding_rs::ISO_8859_7.encode(&value);
        self.inner.extend_from_slice(&output);
        self
    }

    /// Builder function to convert a rust native string into Cyrillic and push onto inner, prefixed with formatting commands.
    pub fn c(mut self, value: String) -> Self {
        self.inner.extend_from_slice(&[b'^', b'C']);
        let (output, _encoding, _had_errors) = encoding_rs::ISO_8859_5.encode(&value);
        self.inner.extend_from_slice(&output);
        self
    }

    /// Builder function to convert a rust native string into SHIFT_JIS and push onto inner, prefixed with formatting commands.
    pub fn j(mut self, value: String) -> Self {
        self.inner.extend_from_slice(&[b'^', b'J']);
        let (output, _encoding, _had_errors) = encoding_rs::SHIFT_JIS.encode(&value);
        self.inner.extend_from_slice(&output);
        self
    }

    /// Builder function to convert a rust native string into ISO_8859_2 and push onto inner, prefixed with formatting commands
    pub fn e(mut self, value: String) -> Self {
        self.inner.extend_from_slice(&[b'^', b'E']);
        let (output, _encoding, _had_errors) = encoding_rs::ISO_8859_2.encode(&value);
        self.inner.extend_from_slice(&output);
        self
    }

    /// Builder function to convert a rust native string into WINDOWS_1254 and push onto inner, prefixed with formatting commands
    pub fn t(mut self, value: String) -> Self {
        self.inner.extend_from_slice(&[b'^', b'T']);
        let (output, _encoding, _had_errors) = encoding_rs::WINDOWS_1254.encode(&value);
        self.inner.extend_from_slice(&output);
        self
    }

    /// Builder function to convert a rust native string into ISO_8859_4 and push onto inner, prefixed with formatting commands
    pub fn b(mut self, value: String) -> Self {
        self.inner.extend_from_slice(&[b'^', b'B']);
        let (output, _encoding, _had_errors) = encoding_rs::ISO_8859_4.encode(&value);
        self.inner.extend_from_slice(&output);
        self
    }

    /// Builder function to convert a rust native string into BIG5 and push onto inner, prefixed with formatting commands
    pub fn h(mut self, value: String) -> Self {
        self.inner.extend_from_slice(&[b'^', b'H']);
        let (output, _encoding, _had_errors) = encoding_rs::BIG5.encode(&value);
        self.inner.extend_from_slice(&output);
        self
    }

    /// Builder function to convert a rust native string into GBK and push onto inner, prefixed with formatting commands
    pub fn s(mut self, value: String) -> Self {
        self.inner.extend_from_slice(&[b'^', b'S']);
        let (output, _encoding, _had_errors) = encoding_rs::GBK.encode(&value);
        self.inner.extend_from_slice(&output);
        self
    }

    /// Builder function to convert a rust native into ISO_8859_7 and push onto inner, prefixed with formatting commands
    pub fn k(mut self, value: String) -> Self {
        self.inner.extend_from_slice(&[b'^', b'K']);
        let (output, _encoding, _had_errors) = encoding_rs::ISO_8859_7.encode(&value);
        self.inner.extend_from_slice(&output);
        self
    }
}
