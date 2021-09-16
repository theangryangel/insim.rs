use deku::prelude::*;
use deku::ctx::{Size, Limit};
use deku::bitvec::{BitSlice, BitVec, Msb0};

fn lfs_string_read(
    rest: &BitSlice<Msb0, u8>,
    bit_size: Size,
) -> Result<(&BitSlice<Msb0, u8>, String), DekuError> {
    // TODO tidy up error handling

    let (rest, mut value) = Vec::read(rest, Limit::new_size(bit_size))?;
    let mut i = 0;

    while i < value.len() {
        if value[i] == 0 {
            break;
        }

        i += 1;
    }

    // TODO: Handle encoding from codepages to utf-8
    // This should probably be a custom type and just implement the deku read/write traits.

    Ok((
        rest,
        String::from_utf8(
            value[..i].to_vec()).map_err(|e| DekuError::Parse(e.to_string())
        )?
    ))
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
#[deku(ctx = "endian: deku::ctx::Endian")]
pub struct RelayHostInfo {
    #[deku(
        reader = "lfs_string_read(deku::rest, Size::Bytes(32))",
        writer = "lfs_string_write(deku::output, &hname, Size::Bytes(32))"
    )]
    hname: String,

    #[deku(
        reader = "lfs_string_read(deku::rest, Size::Bytes(6))",
        writer = "lfs_string_write(deku::output, &track, Size::Bytes(6))"
    )]
    track: String,

    #[deku(bytes = "1")]
    flags: u8,

    #[deku(bytes = "1")]
    numconns: u8,
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
        #[deku(
            reader="lfs_string_read(deku::rest, Size::Bytes(16))",
            writer="lfs_string_write(deku::output, &password, Size::Bytes(16))"
        )]
        password: String,
        #[deku(
            reader="lfs_string_read(deku::rest, Size::Bytes(16))",
            writer="lfs_string_write(deku::output, &name, Size::Bytes(16))"
        )]
        name: String,
    },

    #[deku(id="3")]
    TINY {
        #[deku(bytes = "1")]
        reqi: u8,
        #[deku(bytes = "1")]
        subtype: u8,
    },

    #[deku(id="250")]
    RELAY_ARQ {
        #[deku(bytes = "1")]
        reqi: u8,
        #[deku(bytes="1")]
        sp0: u8,
    },

    #[deku(id="251")]
    RELAY_ARP {
        #[deku(bytes = "1")]
        reqi: u8,
        #[deku(bytes="1")]
        admin: u8,
    },

    #[deku(id="252")]
    RELAY_HLR {
        #[deku(bytes = "1")]
        reqi: u8,
        #[deku(bytes="1")]
        sp0: u8,
    },

    #[deku(id="253")]
    RELAY_HOS {
        #[deku(bytes = "1")]
        reqi: u8,

        #[deku(bytes="1")]
        numhosts: u8,

        #[deku(count = "numhosts")]
        hinfo: Vec<RelayHostInfo>,
    },

    #[deku(id="254")]
    RELAY_SEL {
        #[deku(bytes = "1")]
        reqi: u8,
        #[deku(bytes="1")]
        zero: u8,

        #[deku(
            reader="lfs_string_read(deku::rest, Size::Bytes(32))",
            writer="lfs_string_write(deku::output, &hname, Size::Bytes(32))"
        )]
        hname: String,
        #[deku(
            reader="lfs_string_read(deku::rest, Size::Bytes(16))",
            writer="lfs_string_write(deku::output, &admin, Size::Bytes(16))"
        )]
        admin: String,
        #[deku(
            reader="lfs_string_read(deku::rest, Size::Bytes(16))",
            writer="lfs_string_write(deku::output, &spec, Size::Bytes(16))"
        )]
        spec: String,
    },

    #[deku(id="255")]
    RELAY_ERR {
        #[deku(bytes = "1")]
        reqi: u8,
        #[deku(bytes="1")]
        errno: u8,
    },

}
