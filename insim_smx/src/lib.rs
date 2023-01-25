use insim_core::{prelude::*, point::Point};

#[derive(Debug, InsimDecode, Default, Clone)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Debug, InsimDecode, Default, Clone)]
pub struct Argb {
    pub a: u8,
    pub rgb: Rgb,
}

#[derive(Debug, InsimDecode, Default, Clone)]
pub struct ObjectPoint {
    pub xyz: Point<i32>,
    pub colour: Argb,
}

#[derive(Debug, InsimDecode, Default, Clone)]
pub struct Triangle {
    pub a: u16, // index of the objectpoint
    pub b: u16,
    #[insim(pad_bytes_after = "2")]
    pub c: u16,
}

#[derive(Debug, InsimDecode, Default, Clone)]
pub struct Object {
    pub center: Point<i32>,
    pub radius: i32,
    pub num_object_points: i32,
    pub num_triangles: i32,

    #[insim(count = "num_object_points")]
    pub points: Vec<ObjectPoint>,

    #[insim(count = "num_triangles")]
    pub triangles: Vec<Triangle>,
}

#[derive(Debug, InsimDecode, Default, Clone)]
#[insim(magic = b"LFSSMX")]
pub struct Smx {
    pub game_version: u8,
    pub game_revision: u8,

    pub smx_version: u8,

    pub dimensions: u8,
    pub resolution: u8,

    #[insim(pad_bytes_after = "4")]
    pub vertex_colours: u8,

    #[insim(bytes = "32")]
    pub track: String,

    #[insim(pad_bytes_after = "9")]
    pub ground_colour: Rgb,

    pub num_objects: i32,

    #[insim(count = "num_objects")]
    pub objects: Vec<Object>,

    pub num_checkpoints: i32,

    #[insim(count = "num_checkpoints")]
    pub checkpoint_object_index: Vec<i32>,
}
