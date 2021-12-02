use deku::bitvec::{BitSlice, BitVec, Msb0};
use deku::{ctx::*, DekuError, DekuRead, DekuWrite};

use super::InsimString;

impl DekuWrite<(Endian, Size)> for InsimString {
    fn write(
        &self,
        output: &mut BitVec<Msb0, u8>,
        (_endian, bit_size): (Endian, Size),
    ) -> Result<(), DekuError> {
        // FIXME: implement endian handling
        let value = self.into_insim(bit_size.byte_size().unwrap());
        value.write(output, ())
    }
}

impl DekuWrite<Size> for InsimString {
    fn write(&self, output: &mut BitVec<Msb0, u8>, bit_size: Size) -> Result<(), DekuError> {
        let value = self.into_insim(bit_size.byte_size().unwrap());
        value.write(output, ())
    }
}

impl DekuWrite for InsimString {
    fn write(&self, _output: &mut BitVec<Msb0, u8>, _: ()) -> Result<(), DekuError> {
        panic!("Cannot write an unbounded InsimString!")
    }
}

impl DekuRead<'_, Size> for InsimString {
    fn read(
        input: &BitSlice<Msb0, u8>,
        size: Size,
    ) -> Result<(&BitSlice<Msb0, u8>, Self), DekuError> {
        let (rest, value) = Vec::read(input, Limit::new_size(size))?;

        Ok((rest, InsimString::from_insim(value)))
    }
}

impl DekuRead<'_, (Endian, Size)> for InsimString {
    fn read(
        input: &BitSlice<Msb0, u8>,
        (_endian, size): (Endian, Size),
    ) -> Result<(&BitSlice<Msb0, u8>, Self), DekuError> {
        // FIXME: implement endian handling
        let (rest, value) = Vec::read(input, Limit::new_size(size))?;

        Ok((rest, InsimString::from_insim(value)))
    }
}
