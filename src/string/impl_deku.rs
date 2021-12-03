use deku::bitvec::{BitSlice, BitVec, Msb0};
use deku::{ctx::*, DekuError, DekuRead, DekuWrite};

use super::InsimString;

impl DekuWrite<(Endian, Size)> for InsimString {
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

impl DekuWrite<Size> for InsimString {
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

impl DekuWrite for InsimString {
    fn write(&self, output: &mut BitVec<Msb0, u8>, _: ()) -> Result<(), DekuError> {
        let value = self.into_bytes();
        value.write(output, ())
    }
}

impl DekuRead<'_, Size> for InsimString {
    fn read(
        input: &BitSlice<Msb0, u8>,
        size: Size,
    ) -> Result<(&BitSlice<Msb0, u8>, Self), DekuError> {
        let (rest, value) = Vec::read(input, Limit::new_size(size))?;

        Ok((rest, InsimString::from_bytes(value)))
    }
}

impl DekuRead<'_, (Endian, Size)> for InsimString {
    fn read(
        input: &BitSlice<Msb0, u8>,
        (_endian, size): (Endian, Size),
    ) -> Result<(&BitSlice<Msb0, u8>, Self), DekuError> {
        // FIXME: implement endian handling
        let (rest, value) = Vec::read(input, Limit::new_size(size))?;

        Ok((rest, InsimString::from_bytes(value)))
    }
}
