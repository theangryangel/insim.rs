//! Utilities for working with strings from Insim.

use super::{escape, strip_trailing_nul, unescape};
use encoding_rs;
use itertools::Itertools;
use std::vec::Vec;

const CODEPAGES: &[u8] = &[b'L', b'G', b'J', b'E', b'T', b'B', b'H', b'S', b'K', b'9'];
const CODEPAGE_MARKER: u8 = b'^';

/// A representation of the format-able wire format of a LFS "codepage string".
///
/// LFS strings are a sequence of u8 bytes, with an optional trailing \0.
/// The "format-able string" may be compromised of characters from multiple code pages, indicated by a `^` and
/// a following code page identifier character. i.e. `^L` indicates Latin1.
///
/// The common practise is to use the function `to_lossy_string` to convert to a standard Rust
/// String.
///
/// The struct also supports a 'builder' style interface for creating the raw bytes.
#[derive(PartialEq, Default, Debug)]
pub struct ICodepageString {
    pub(crate) inner: Vec<u8>,
}

impl ICodepageString {
    /// Create a new empty IString.
    /// This will not allocate until elements are pushed onto it.
    pub const fn new() -> Self {
        Self { inner: Vec::new() }
    }

    /// Creates a new emptyy `IString` with the specified capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: Vec::with_capacity(capacity),
        }
    }

    /// Returns the length of the inner bytes.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns `true` if the string is empty.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Clear the inner vec
    pub fn clear(&mut self) {
        self.inner.clear()
    }

    /// Takes a slice of u8, strips any trailing \0 and returns an IString.
    pub fn from_bytes(input: &[u8]) -> Self {
        ICodepageString {
            inner: strip_trailing_nul(input).to_vec(),
        }
    }

    /// Returns a slice of the inner bytes.
    pub fn into_bytes(&self) -> &[u8] {
        &self.inner
    }

    /// Convert from a String using the default LFS encoding ("Latin1")
    pub fn from_string(value: String) -> ICodepageString {
        let (output, _encoding, _had_errors) = encoding_rs::WINDOWS_1252.encode(&value);
        ICodepageString {
            inner: escape(&output.to_vec()),
        }
    }

    /// Convert a InsimString into a native rust String, with potential lossy conversion from codepages
    pub fn to_lossy_string(&self) -> String {
        // TODO: Do we want to unescape first? Or do we escape after conversion?
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

        // This pre-allocation is the best guess we can make here
        let mut result = String::with_capacity(self.len());

        for pair in indices.windows(2) {
            let range = &input[pair[0]..pair[1]];

            if range.len() < 2 {
                result.push_str(&String::from_utf8(unescape(range).to_vec()).unwrap());
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
                        encoding_rs::WINDOWS_1251.decode(&range[2..]);
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
                        encoding_rs::ISO_8859_13.decode(&range[2..]);
                    cow.to_string()
                }

                [b'^', b'H'] => {
                    let (cow, _encoding_used, _had_errors) = encoding_rs::GBK.decode(&range[2..]);
                    cow.to_string()
                }

                [b'^', b'S'] => {
                    let (cow, _encoding_used, _had_errors) =
                        encoding_rs::EUC_KR.decode(&range[2..]);
                    cow.to_string()
                }

                [b'^', b'K'] => {
                    let (cow, _encoding_used, _had_errors) = encoding_rs::BIG5.decode(&range[2..]);
                    cow.to_string()
                }

                [b'^', b'L'] => {
                    let (cow, _encoding, _had_errors) =
                        encoding_rs::WINDOWS_1252.decode(&range[2..]);
                    cow.to_string()
                }

                [b'^', b'9'] => {
                    // ^9 is a special case, it represents reset to default codepage, AND also
                    // reset to default colour. We cannot just strip the ^9.
                    let (cow, _encoding, _had_errors) = encoding_rs::WINDOWS_1252.decode(range);
                    cow.to_string()
                }

                // Latin fallback, include the prefix
                _ => {
                    let (cow, _encoding, _had_errors) = encoding_rs::WINDOWS_1252.decode(range);
                    cow.to_string()
                }
            };

            result.push_str(&decoded);
        }

        String::from_utf8(unescape(result.as_bytes())).unwrap()
    }

    // FIXME: Builder functions should escape the input

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

// XXX: I've left this and the IString impls seperate for now as I suspect there are use cases
// where implementing them via a common trait may be problematic. I'm not sure what they are
// *yet*, so I'm leaving them effectively duplicated, for now.

use std::fmt;

impl Clone for ICodepageString {
    fn clone(&self) -> Self {
        ICodepageString {
            inner: self.inner.clone(),
        }
    }

    fn clone_from(&mut self, source: &Self) {
        self.inner.clone_from(&source.inner);
    }
}

impl From<String> for ICodepageString {
    #[inline]
    fn from(s: String) -> Self {
        ICodepageString::from_string(s)
    }
}

impl From<&str> for ICodepageString {
    #[inline]
    fn from(s: &str) -> Self {
        ICodepageString::from_string(s.into())
    }
}

impl fmt::Display for ICodepageString {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_lossy_string())
    }
}

#[cfg(feature = "serde")]
use serde::ser::{Serialize, Serializer};

#[cfg(feature = "serde")]
impl Serialize for ICodepageString {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

use deku::bitvec::{BitSlice, BitVec, Msb0};
use deku::{ctx::*, DekuError, DekuRead, DekuWrite};

impl DekuWrite<(Endian, Size)> for ICodepageString {
    fn write(
        &self,
        output: &mut BitVec<Msb0, u8>,
        (_endian, bit_size): (Endian, Size),
    ) -> Result<(), DekuError> {
        // FIXME: Handle Endian
        let orig_size = output.len();
        if self.is_empty() {
            output.resize(orig_size + bit_size.bit_size(), false);
            return Ok(());
        }

        let max_size = bit_size.byte_size().unwrap();
        let input_size = if self.len() < max_size {
            self.len()
        } else {
            max_size
        };

        let res = (&self.into_bytes()[0..input_size]).write(output, ());
        if let Err(e) = res {
            return Err(e);
        }
        if input_size != max_size {
            output.resize(orig_size + bit_size.bit_size(), false);
        }

        Ok(())
    }
}

impl DekuWrite<Size> for ICodepageString {
    fn write(&self, output: &mut BitVec<Msb0, u8>, bit_size: Size) -> Result<(), DekuError> {
        let orig_size = output.len();
        if self.is_empty() {
            output.resize(orig_size + bit_size.bit_size(), false);
            return Ok(());
        }
        let max_size = bit_size.byte_size().unwrap();
        let input_size = if self.len() < max_size {
            self.len()
        } else {
            max_size
        };

        let res = (&self.into_bytes()[0..input_size]).write(output, ());
        if let Err(e) = res {
            return Err(e);
        }
        if input_size != max_size {
            output.resize(orig_size + bit_size.bit_size(), false);
        }

        Ok(())
    }
}

impl DekuWrite for ICodepageString {
    fn write(&self, output: &mut BitVec<Msb0, u8>, _: ()) -> Result<(), DekuError> {
        let value = self.into_bytes();
        value.write(output, ())
    }
}

impl DekuRead<'_, Size> for ICodepageString {
    fn read(
        input: &BitSlice<Msb0, u8>,
        size: Size,
    ) -> Result<(&BitSlice<Msb0, u8>, Self), DekuError> {
        let (rest, value) = Vec::read(input, Limit::new_size(size))?;

        Ok((rest, ICodepageString::from_bytes(&value)))
    }
}

impl DekuRead<'_, (Endian, Size)> for ICodepageString {
    fn read(
        input: &BitSlice<Msb0, u8>,
        (_endian, size): (Endian, Size),
    ) -> Result<(&BitSlice<Msb0, u8>, Self), DekuError> {
        // FIXME: implement endian handling
        let (rest, value) = Vec::read(input, Limit::new_size(size))?;

        Ok((rest, ICodepageString::from_bytes(&value)))
    }
}
