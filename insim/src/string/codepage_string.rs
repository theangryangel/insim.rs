//! Utilities for working with 'Codepage strings' from Insim.

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
#[derive(PartialEq, Eq, Default, Debug)]
pub struct CodepageString {
    pub(crate) inner: Vec<u8>,
}

impl CodepageString {
    /// Create a new empty CodepageString.
    /// This will not allocate until elements are pushed onto it.
    pub const fn new() -> Self {
        Self { inner: Vec::new() }
    }

    /// Creates a new emptyy `CodepageString` with the specified capacity.
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

    /// Takes a slice of u8, strips any trailing \0 and returns an CodepageString.
    pub fn from_bytes(input: &[u8]) -> Self {
        CodepageString {
            inner: strip_trailing_nul(input).to_vec(),
        }
    }

    /// Returns a slice of the inner bytes.
    pub fn into_bytes(&self) -> &[u8] {
        &self.inner
    }

    /// Convert from a String using the default LFS encoding ("Latin1")
    pub fn from_string(value: String) -> CodepageString {
        let (output, _encoding, _had_errors) = encoding_rs::WINDOWS_1252.encode(&value);
        CodepageString {
            inner: escape(&output),
        }
    }

    /// Convert a InsimString into a native rust String, with potential lossy conversion from codepages
    pub fn to_lossy_string(&self) -> String {
        // FIXME this should all be using Cow

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
        let mut result = String::with_capacity(self.len());

        for pair in indices.windows(2) {
            let range = &input[pair[0]..pair[1]];

            if range.len() < 2 {
                result.push_str(&String::from_utf8_lossy(&unescape(range)));
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

        String::from_utf8_lossy(&unescape(result.as_bytes())).into()
    }

    /// Builder function to convert a rust native string into Latin1 and push onto inner, prefixed with formatting commands.
    pub fn latin1(mut self, value: String) -> Self {
        self.inner.extend_from_slice(&[b'^', b'L']);
        let (output, _encoding, _had_errors) = encoding_rs::WINDOWS_1252.encode(&value);
        self.inner.extend_from_slice(&escape(&output));
        self
    }

    /// Builder function to convert a rust native string into Greek and push onto inner, prefixed with formatting commands.
    pub fn greek(mut self, value: String) -> Self {
        self.inner.extend_from_slice(&[b'^', b'G']);
        let (output, _encoding, _had_errors) = encoding_rs::ISO_8859_7.encode(&value);
        self.inner.extend_from_slice(&escape(&output));
        self
    }

    /// Builder function to convert a rust native string into Cyrillic and push onto inner, prefixed with formatting commands.
    pub fn cyrillic(mut self, value: String) -> Self {
        self.inner.extend_from_slice(&[b'^', b'C']);
        let (output, _encoding, _had_errors) = encoding_rs::ISO_8859_5.encode(&value);
        self.inner.extend_from_slice(&escape(&output));
        self
    }

    /// Builder function to convert a rust native string into SHIFT_JIS and push onto inner, prefixed with formatting commands.
    pub fn japanese(mut self, value: String) -> Self {
        self.inner.extend_from_slice(&[b'^', b'J']);
        let (output, _encoding, _had_errors) = encoding_rs::SHIFT_JIS.encode(&value);
        self.inner.extend_from_slice(&escape(&output));
        self
    }

    /// Builder function to convert a rust native string into ISO_8859_2 and push onto inner, prefixed with formatting commands
    pub fn central_european(mut self, value: String) -> Self {
        self.inner.extend_from_slice(&[b'^', b'E']);
        let (output, _encoding, _had_errors) = encoding_rs::ISO_8859_2.encode(&value);
        self.inner.extend_from_slice(&escape(&output));
        self
    }

    /// Builder function to convert a rust native string into WINDOWS_1254 and push onto inner, prefixed with formatting commands
    pub fn turkish(mut self, value: String) -> Self {
        self.inner.extend_from_slice(&[b'^', b'T']);
        let (output, _encoding, _had_errors) = encoding_rs::WINDOWS_1254.encode(&value);
        self.inner.extend_from_slice(&escape(&output));
        self
    }

    /// Builder function to convert a rust native string into ISO_8859_4 and push onto inner, prefixed with formatting commands
    pub fn baltic(mut self, value: String) -> Self {
        self.inner.extend_from_slice(&[b'^', b'B']);
        let (output, _encoding, _had_errors) = encoding_rs::ISO_8859_4.encode(&value);
        self.inner.extend_from_slice(&escape(&output));
        self
    }

    /// Builder function to convert a rust native string into BIG5 and push onto inner, prefixed with formatting commands
    pub fn traditional_chinese(mut self, value: String) -> Self {
        self.inner.extend_from_slice(&[b'^', b'H']);
        let (output, _encoding, _had_errors) = encoding_rs::BIG5.encode(&value);
        self.inner.extend_from_slice(&escape(&output));
        self
    }

    /// Builder function to convert a rust native string into GBK and push onto inner, prefixed with formatting commands
    pub fn simplified_chinese(mut self, value: String) -> Self {
        self.inner.extend_from_slice(&[b'^', b'S']);
        let (output, _encoding, _had_errors) = encoding_rs::GBK.encode(&value);
        self.inner.extend_from_slice(&escape(&output));
        self
    }

    /// Builder function to convert a rust native into ISO_8859_7 and push onto inner, prefixed with formatting commands
    pub fn korean(mut self, value: String) -> Self {
        self.inner.extend_from_slice(&[b'^', b'K']);
        let (output, _encoding, _had_errors) = encoding_rs::ISO_8859_7.encode(&value);
        self.inner.extend_from_slice(&escape(&output));
        self
    }

    /// Build function to push black onto inner
    pub fn black(mut self) -> Self {
        self.inner.extend_from_slice(&[b'^', b'0']);
        self
    }

    /// Build function to push red onto inner
    pub fn red(mut self) -> Self {
        self.inner.extend_from_slice(&[b'^', b'1']);
        self
    }

    /// Build function to push yellow onto inner
    pub fn yellow(mut self) -> Self {
        self.inner.extend_from_slice(&[b'^', b'2']);
        self
    }

    /// Build function to push blue onto inner
    pub fn blue(mut self) -> Self {
        self.inner.extend_from_slice(&[b'^', b'4']);
        self
    }

    /// Build function to push purple onto inner
    pub fn purple(mut self) -> Self {
        self.inner.extend_from_slice(&[b'^', b'5']);
        self
    }

    /// Build function to push light blue onto inner
    pub fn light_blue(mut self) -> Self {
        self.inner.extend_from_slice(&[b'^', b'6']);
        self
    }

    /// Build function to push white onto inner
    pub fn white(mut self) -> Self {
        self.inner.extend_from_slice(&[b'^', b'7']);
        self
    }

    /// Build function to push default colour onto inner
    pub fn default_colour(mut self) -> Self {
        self.inner.extend_from_slice(&[b'^', b'8']);
        self
    }

    /// Americanised version of default_colour
    pub fn default_color(self) -> Self {
        self.default_colour()
    }

    /// Build function to push default colour & codepage onto inner
    pub fn default_codepage_and_colour(mut self) -> Self {
        self.inner.extend_from_slice(&[b'^', b'9']);
        self
    }

    /// Americanised version of default_codepage_and_colour
    pub fn default_codepage_and_color(self) -> Self {
        self.default_codepage_and_colour()
    }
}

use std::fmt;

impl Clone for CodepageString {
    fn clone(&self) -> Self {
        CodepageString {
            inner: self.inner.clone(),
        }
    }

    fn clone_from(&mut self, source: &Self) {
        self.inner.clone_from(&source.inner);
    }
}

impl From<String> for CodepageString {
    #[inline]
    fn from(s: String) -> Self {
        CodepageString::from_string(s)
    }
}

impl From<&str> for CodepageString {
    #[inline]
    fn from(s: &str) -> Self {
        CodepageString::from_string(s.into())
    }
}

impl fmt::Display for CodepageString {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_lossy_string())
    }
}

#[cfg(feature = "serde")]
use serde::ser::{Serialize, Serializer};

#[cfg(feature = "serde")]
impl Serialize for CodepageString {
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

impl DekuWrite<(Endian, Size)> for CodepageString {
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

        let max_size = bit_size.byte_size()?;
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

impl DekuWrite<Size> for CodepageString {
    fn write(&self, output: &mut BitVec<Msb0, u8>, bit_size: Size) -> Result<(), DekuError> {
        let orig_size = output.len();
        if self.is_empty() {
            output.resize(orig_size + bit_size.bit_size(), false);
            return Ok(());
        }
        let max_size = bit_size.byte_size()?;
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

impl DekuWrite<Endian> for CodepageString {
    fn write(&self, output: &mut BitVec<Msb0, u8>, _endian: Endian) -> Result<(), DekuError> {
        // FIXME endian
        let value = self.into_bytes();
        value.write(output, ())
    }
}

impl DekuWrite for CodepageString {
    fn write(&self, output: &mut BitVec<Msb0, u8>, _: ()) -> Result<(), DekuError> {
        let value = self.into_bytes();
        value.write(output, ())
    }
}

impl DekuRead<'_, Size> for CodepageString {
    fn read(
        input: &BitSlice<Msb0, u8>,
        size: Size,
    ) -> Result<(&BitSlice<Msb0, u8>, Self), DekuError> {
        let (rest, value) = Vec::read(input, Limit::new_size(size))?;

        Ok((rest, CodepageString::from_bytes(&value)))
    }
}

impl DekuRead<'_, (Endian, Size)> for CodepageString {
    fn read(
        input: &BitSlice<Msb0, u8>,
        (_endian, size): (Endian, Size),
    ) -> Result<(&BitSlice<Msb0, u8>, Self), DekuError> {
        // FIXME: implement endian handling
        let (rest, value) = Vec::read(input, Limit::new_size(size))?;

        Ok((rest, CodepageString::from_bytes(&value)))
    }
}
