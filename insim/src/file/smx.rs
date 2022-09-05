use crate::protocol::position::Point;
use crate::string::istring;

use deku::prelude::*;

#[derive(Debug, DekuRead, DekuWrite)]
#[deku(
    ctx = "endian: deku::ctx::Endian",
    ctx_default = "deku::ctx::Endian::Little",
    endian = "endian"
)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Debug, DekuRead, DekuWrite)]
#[deku(
    ctx = "endian: deku::ctx::Endian",
    ctx_default = "deku::ctx::Endian::Little",
    endian = "endian"
)]
pub struct Argb {
    pub a: u8,
    pub rgb: Rgb,
}

#[derive(Debug, DekuRead, DekuWrite)]
#[deku(
    ctx = "endian: deku::ctx::Endian",
    ctx_default = "deku::ctx::Endian::Little",
    endian = "endian"
)]
pub struct ObjectPoint {
    pub xyz: Point<i32>,
    pub colour: Argb,
}

#[derive(Debug, DekuRead, DekuWrite)]
#[deku(
    ctx = "endian: deku::ctx::Endian",
    ctx_default = "deku::ctx::Endian::Little",
    endian = "endian"
)]
pub struct Triangle {
    pub a: u16, // index of the objectpoint
    pub b: u16,
    #[deku(pad_bytes_after = "2")]
    pub c: u16,
}

#[derive(Debug, DekuRead, DekuWrite)]
#[deku(
    ctx = "endian: deku::ctx::Endian",
    ctx_default = "deku::ctx::Endian::Little",
    endian = "endian"
)]
pub struct Object {
    pub center: Point<i32>,
    pub radius: i32,
    pub num_object_points: i32,
    pub num_triangles: i32,

    #[deku(count = "num_object_points")]
    pub points: Vec<ObjectPoint>,

    #[deku(count = "num_triangles")]
    pub triangles: Vec<Triangle>,
}

#[derive(Debug, DekuRead, DekuWrite)]
#[deku(
    magic = b"LFSSMX",
    ctx = "endian: deku::ctx::Endian",
    ctx_default = "deku::ctx::Endian::Little",
    endian = "endian"
)]
pub struct Smx {
    pub game_version: u8,
    pub game_revision: u8,

    pub smx_version: u8,

    pub dimensions: u8,
    pub resolution: u8,

    #[deku(pad_bytes_after = "4")]
    pub vertex_colours: u8,

    #[deku(
        reader = "istring::read(deku::rest, 32)",
        writer = "istring::write(deku::output, &self.track, 32)"
    )]
    pub track: String,

    #[deku(pad_bytes_after = "9")]
    pub ground_colour: Rgb,

    pub num_objects: i32,

    #[deku(count = "num_objects")]
    pub objects: Vec<Object>,

    pub num_checkpoints: i32,

    #[deku(count = "num_checkpoints")]
    pub checkpoint_object_index: Vec<i32>,
}
