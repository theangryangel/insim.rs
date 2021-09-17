use deku::bitvec::{BitSlice, BitVec, Msb0};
use deku::ctx::{Limit, Size};
use deku::prelude::*;

use crate::InsimString;

fn lfs_string_read(
    rest: &BitSlice<Msb0, u8>,
    bit_size: Size,
) -> Result<(&BitSlice<Msb0, u8>, String), DekuError> {
    // TODO tidy up error handling
    let (rest, value) = Vec::read(rest, Limit::new_size(bit_size))?;

    return Ok((
        rest,
        String::from_lfs(value).map_err(|e| DekuError::Parse(e.to_string()))?,
    ));
}

/// Parse from String to u8 and write
fn lfs_string_write(
    output: &mut BitVec<Msb0, u8>,
    field: &str,
    bit_size: Size,
) -> Result<(), DekuError> {
    let size = bit_size.byte_size().unwrap();

    let value = field
        .to_string()
        .to_lfs(size)
        .map_err(|e| DekuError::Parse(e.to_string()))?;
    value.write(output, ())
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
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
#[deku(ctx = "_endian: deku::ctx::Endian")]
pub struct MultiCarInfoCompCar {
    #[deku(bytes = "2")]
    node: u16,

    #[deku(bytes = "2")]
    lap: u16,

    #[deku(bytes = "1")]
    plid: u8,

    #[deku(bytes = "1")]
    position: u8,

    #[deku(bytes = "1", pad_bytes_after = "1")]
    info: u8,

    // sp3 handled by pad_bytes_after
    #[deku(bytes = "4")]
    x: i32, // X map (65536 = 1 metre)

    #[deku(bytes = "4")]
    y: i32, // Y map (65536 = 1 metre)

    #[deku(bytes = "4")]
    z: i32, // Z alt (65536 = 1 metre)

    #[deku(bytes = "2")]
    speed: u16, // speed (32768 = 100 m/s)

    #[deku(bytes = "2")]
    direction: u16, // direction of car's motion : 0 = world y direction, 32768 = 180 deg

    #[deku(bytes = "2")]
    heading: u16, // direction of forward axis : 0 = world y direction, 32768 = 180 deg

    #[deku(bytes = "2")]
    angvel: i16, // signed, rate of change of heading : (16384 = 360 deg/s)
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "big", type = "u8")]
pub enum Insim {
    // TODO The rest of the packets
    #[deku(id = "1")]
    Init {
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
            reader = "lfs_string_read(deku::rest, Size::Bytes(16))",
            writer = "lfs_string_write(deku::output, &password, Size::Bytes(16))"
        )]
        password: String,
        #[deku(
            reader = "lfs_string_read(deku::rest, Size::Bytes(16))",
            writer = "lfs_string_write(deku::output, &name, Size::Bytes(16))"
        )]
        name: String,
    },

    #[deku(id = "2")]
    Version {
        #[deku(bytes = "1", pad_bytes_after = "1")]
        reqi: u8,

        #[deku(
            reader = "lfs_string_read(deku::rest, Size::Bytes(8))",
            writer = "lfs_string_write(deku::output, &version, Size::Bytes(8))"
        )]
        version: String,

        #[deku(
            reader = "lfs_string_read(deku::rest, Size::Bytes(8))",
            writer = "lfs_string_write(deku::output, &product, Size::Bytes(8))"
        )]
        product: String,

        #[deku(bytes = "2")]
        insimver: u16,
    },

    #[deku(id = "3")]
    Tiny {
        #[deku(bytes = "1")]
        reqi: u8,
        #[deku(bytes = "1")]
        subtype: u8,
    },

    #[deku(id = "11")]
    MessageOut {
        #[deku(bytes = "1", pad_bytes_after = "1")]
        reqi: u8,

        #[deku(bytes = "1")]
        ucid: u8,

        #[deku(bytes = "1")]
        plid: u8,

        #[deku(bytes = "1")]
        usertype: u8,

        #[deku(bytes = "1")]
        textstart: u8,

        #[deku(
            reader = "lfs_string_read(deku::rest, Size::Bytes(128))",
            writer = "lfs_string_write(deku::output, &msg, Size::Bytes(128))"
        )]
        msg: String,
    },

    #[deku(id = "38")]
    MultiCarInfo {
        #[deku(bytes = "1")]
        reqi: u8,
        #[deku(bytes = "1")]
        numc: u8,

        #[deku(count = "numc")]
        info: Vec<MultiCarInfoCompCar>,
    },

    #[deku(id = "250")]
    RelayAdminRequest {
        #[deku(bytes = "1", pad_bytes_after = "1")]
        reqi: u8,
        // sp0 is handled by pad_bytes_after in reqi
    },

    #[deku(id = "251")]
    RelayAdminResponse {
        #[deku(bytes = "1")]
        reqi: u8,
        #[deku(bytes = "1")]
        admin: u8,
    },

    #[deku(id = "252")]
    RelayHostListRequest {
        #[deku(bytes = "1", pad_bytes_after = "1")]
        reqi: u8,
        // sp0 is handled by pad_bytes_after in reqi
    },

    #[deku(id = "253")]
    RelayHostList {
        #[deku(bytes = "1")]
        reqi: u8,

        #[deku(bytes = "1")]
        numhosts: u8,

        #[deku(count = "numhosts")]
        hinfo: Vec<RelayHostInfo>,
    },

    #[deku(id = "254")]
    RelaySelect {
        #[deku(bytes = "1", pad_bytes_after = "1")]
        reqi: u8,

        // zero handled by pad_bytes_after
        #[deku(
            reader = "lfs_string_read(deku::rest, Size::Bytes(32))",
            writer = "lfs_string_write(deku::output, &hname, Size::Bytes(32))"
        )]
        hname: String,
        #[deku(
            reader = "lfs_string_read(deku::rest, Size::Bytes(16))",
            writer = "lfs_string_write(deku::output, &admin, Size::Bytes(16))"
        )]
        admin: String,
        #[deku(
            reader = "lfs_string_read(deku::rest, Size::Bytes(16))",
            writer = "lfs_string_write(deku::output, &spec, Size::Bytes(16))"
        )]
        spec: String,
    },

    #[deku(id = "255")]
    RelayErr {
        #[deku(bytes = "1")]
        reqi: u8,
        #[deku(bytes = "1")]
        errno: u8,
    },
}
