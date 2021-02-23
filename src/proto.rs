use deku::prelude::*;
use deku::ctx::Size;

fn lfs_string_read(
    rest: &BitSlice<Msb0, u8>,
    bit_size: Size,
) -> Result<(&BitSlice<Msb0, u8>, String), DekuError> {
    let (rest, value) = u8::read(rest, bit_size)?;
    // TODO remove extra \0 padding
    Ok((rest, value.to_string()))
}

/// Parse from String to u8 and write
fn lfs_string_write(
    output: &mut BitVec<Msb0, u8>,
    field: &str,
    bit_size: Size,
) -> Result<(), DekuError> {
    let size = bit_size.byte_size().unwrap();

    // TODO we can do this without allocating a buffer, etc.
    // Fix this.
    let mut buf = field.as_bytes().to_vec();
    if buf.len() < size {
        buf.reserve(size - buf.len());
        for _i in 0..(size-buf.len()) {
            buf.push(0);
        }
    }

    let value = &buf[0..size];
    value.write(output, ())
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(endian="big", type = "u8")]
pub enum Insim {
    #[deku(id="1")]
    INIT {
        #[deku(bytes = "1")]
        reqi: u8,
        #[deku(bytes = "1")]
        zero: u8,
        #[deku(bytes = "2")]
        udpport: u16,
        #[deku(bytes = "2")]
        flags: u16,
        #[deku(bytes = "1")]
        version: u8,
        #[deku(bytes = "1")]
        prefix: u8,
        #[deku(bytes = "2")]
        interval: u16,
        #[deku(reader="lfs_string_read(deku::rest, Size::Bytes(16))", writer="lfs_string_write(deku::output, &password, Size::Bytes(16))")]
        password: String,
        #[deku(reader="lfs_string_read(deku::rest, Size::Bytes(16))", writer="lfs_string_write(deku::output, &name, Size::Bytes(16))")]
        name: String,
    },

    #[deku(id="3")]
    TINY {
        #[deku(bytes = "1")]
        reqi: u8,
        #[deku(bytes = "1")]
        subtype: u8,
    }
}
