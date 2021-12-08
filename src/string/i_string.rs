use super::{escape, strip_trailing_nul, unescape};
use std::vec::Vec;

/// A representation of the non-format-able wire format of a LFS "string".
///
/// Effectively LFS transmits strings as a "CString", with the exception that a CString must always
/// be terminated by a \0 byte. In LFS's wireformat this is not always the case.
///
/// Typically a ICString is used for simple fields, like plates, tracks, etc. and IUString is used
/// for the more "complex" string which allows a mix of codepages.
#[derive(PartialEq, Default, Debug)]
pub struct IString {
    pub(crate) inner: Vec<u8>,
}

impl IString {
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
        Self {
            inner: strip_trailing_nul(input).to_vec(),
        }
    }

    /// Returns a slice of the inner bytes.
    pub fn into_bytes(&self) -> &[u8] {
        &self.inner
    }

    /// Convert from a String
    pub fn from_string(value: String) -> Self {
        Self {
            inner: escape(value.as_bytes()),
        }
    }

    /// Convert to a String
    pub fn to_lossy_string(&self) -> String {
        String::from_utf8_lossy(&unescape(&self.inner)).to_string()
    }
}

// XXX: I've left this and the ICodePageString impls seperate for now as I suspect there are use cases
// where implementing them via a common trait may be problematic. I'm not sure what they are
// *yet*, so I'm leaving them effectively duplicated, for now.

use std::fmt;

impl Clone for IString {
    fn clone(&self) -> Self {
        IString {
            inner: self.inner.clone(),
        }
    }

    fn clone_from(&mut self, source: &Self) {
        self.inner.clone_from(&source.inner);
    }
}

impl From<String> for IString {
    #[inline]
    fn from(s: String) -> Self {
        IString::from_string(s)
    }
}

impl From<&str> for IString {
    #[inline]
    fn from(s: &str) -> Self {
        IString::from_string(s.into())
    }
}

impl fmt::Display for IString {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_lossy_string())
    }
}

#[cfg(feature = "serde")]
use serde::ser::{Serialize, Serializer};

#[cfg(feature = "serde")]
impl Serialize for IString {
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

impl DekuWrite<(Endian, Size)> for IString {
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

impl DekuWrite<Size> for IString {
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

impl DekuWrite for IString {
    fn write(&self, output: &mut BitVec<Msb0, u8>, _: ()) -> Result<(), DekuError> {
        let value = self.into_bytes();
        value.write(output, ())
    }
}

impl DekuRead<'_, Size> for IString {
    fn read(
        input: &BitSlice<Msb0, u8>,
        size: Size,
    ) -> Result<(&BitSlice<Msb0, u8>, Self), DekuError> {
        let (rest, value) = Vec::read(input, Limit::new_size(size))?;

        Ok((rest, IString::from_bytes(&value)))
    }
}

impl DekuRead<'_, (Endian, Size)> for IString {
    fn read(
        input: &BitSlice<Msb0, u8>,
        (_endian, size): (Endian, Size),
    ) -> Result<(&BitSlice<Msb0, u8>, Self), DekuError> {
        // FIXME: implement endian handling
        let (rest, value) = Vec::read(input, Limit::new_size(size))?;

        Ok((rest, IString::from_bytes(&value)))
    }
}
