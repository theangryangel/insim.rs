//! # insim_smx
//!
//! Parse a Live for Speed smx (Simple Mesh) file.
//!
//! Historically Live for Speed has made SMX files available for each track.
//!
//! For at least Rockingham there is no SMX file and there are no plans to make it
//! available.
//!
//! I would suggest that SMX files should be considered historical at this point.

#[cfg(test)]
use std::io::{Cursor, Read, Seek, SeekFrom};
use std::{
    fs::{self, File},
    io::ErrorKind,
    path::PathBuf,
};

use insim_core::{
    binrw::{self, binrw, BinRead},
    point::Point,
    string::{binrw_parse_codepage_string, binrw_write_codepage_string},
};
use thiserror::Error;

#[non_exhaustive]
#[derive(Error, Debug)]
#[allow(missing_docs)]
pub enum Error {
    #[error("IO Error: {kind}: {message}")]
    IO { kind: ErrorKind, message: String },

    #[error("BinRw Err {0:?}")]
    BinRw(#[from] binrw::Error),
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::IO {
            kind: e.kind(),
            message: e.to_string(),
        }
    }
}

#[binrw]
#[derive(Debug, Default, Clone)]
/// Red Green Blue
pub struct Rgb {
    /// Red
    pub r: u8,
    /// Green
    pub g: u8,
    /// Blue
    pub b: u8,
}

#[binrw]
#[derive(Debug, Default, Clone)]
/// RGB with alpha channel
pub struct Argb {
    /// Alpha
    pub a: u8,
    /// RGB
    pub rgb: Rgb,
}

#[binrw]
#[derive(Debug, Default, Clone)]
/// An Object at a given point with a colour
pub struct ObjectPoint {
    /// Position/point
    pub xyz: Point<i32>,
    /// Colour
    pub colour: Argb,
}

#[binrw]
#[derive(Debug, Default, Clone)]
/// Triangle block
pub struct Triangle {
    /// Vertex A
    pub a: u16,
    /// Vertex B
    pub b: u16,
    #[brw(pad_after = 2)]
    /// Vertex C
    pub c: u16,
}

#[binrw]
#[derive(Debug, Default, Clone)]
/// Object Block
pub struct Object {
    /// Center point of object
    pub center: Point<i32>,
    /// Radius of object
    pub radius: i32,

    #[bw(calc = points.len() as i32)]
    num_object_points: i32,

    #[bw(calc = triangles.len() as i32)]
    num_triangles: i32,

    #[br(count = num_object_points)]
    /// List of points
    pub points: Vec<ObjectPoint>,

    #[br(count = num_triangles)]
    /// list of triangles
    pub triangles: Vec<Triangle>,
}

#[binrw]
#[derive(Debug, Default, Clone)]
#[brw(magic = b"LFSSMX", little)]
/// Smx file
pub struct Smx {
    /// Game version
    pub game_version: u8,
    /// Game revision
    pub game_revision: u8,

    /// SMX file version
    pub smx_version: u8,

    /// Always 3. Usually.
    pub dimensions: u8,
    /// Resolution: 0 = High, 1 = Low
    pub resolution: u8,

    #[brw(pad_after = 4)]
    /// Always 1
    pub vertex_colours: u8,

    #[bw(write_with = binrw_write_codepage_string::<32, _>)]
    #[br(parse_with = binrw_parse_codepage_string::<32, _>)]
    /// Track
    pub track: String,

    #[brw(pad_after = 9)]
    /// Colour of ground
    pub ground_colour: Rgb,

    #[bw(calc = objects.len() as i32)]
    num_objects: i32,

    #[br(count = num_objects)]
    /// List of objects
    pub objects: Vec<Object>,

    #[bw(calc = checkpoint_object_index.len() as i32)]
    num_checkpoints: i32,

    #[br(count = num_checkpoints)]
    /// List of checkpoints
    pub checkpoint_object_index: Vec<i32>,
}

impl Smx {
    /// Read and parse a SMX file into a [Smx] struct.
    pub fn from_file(i: &mut File) -> Result<Self, Error> {
        Self::read(i).map_err(Error::from)
    }

    /// Read and parse a SMX file into a [Smx] struct.
    pub fn from_pathbuf(i: &PathBuf) -> Result<Self, Error> {
        if !i.exists() {
            return Err(Error::IO {
                kind: std::io::ErrorKind::NotFound,
                message: format!("Path {i:?} does not exist"),
            });
        }

        let mut input = fs::File::open(i).map_err(Error::from)?;
        let result = Self::read(&mut input)?;

        Ok(result)
    }
}

#[cfg(test)]
fn assert_valid_autocross_3dh(p: &Smx) {
    assert_eq!(p.objects.len(), 1666);
    assert_eq!(p.checkpoint_object_index.len(), 6);
    assert_eq!(p.track, "Autocross");
    assert_eq!(p.track.as_bytes().len(), 9);
}

#[test]
fn test_smx_decode_from_pathbuf() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("./tests/Autocross_3DH.smx");
    let p = Smx::from_pathbuf(&path).expect("Expected SMX file to be parsed");

    assert_valid_autocross_3dh(&p);
}

#[test]
fn test_smx_decode_from_file() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("./tests/Autocross_3DH.smx");
    let mut file = File::open(path).expect("Expected Autocross_3DH.smx to exist");
    let p = Smx::from_file(&mut file).expect("Expected SMX file to be parsed");

    let pos = file.stream_position().unwrap();
    let end = file.seek(SeekFrom::End(0)).unwrap();

    assert_eq!(pos, end, "Expected the whole file to be completely read");

    assert_valid_autocross_3dh(&p);
}

#[test]
fn test_smx_encode() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("./tests/Autocross_3DH.smx");
    let p = Smx::from_pathbuf(&path).expect("Expected SMX file to be parsed");

    let mut file = File::open(path).expect("Expected Autocross_3DH.smx to exist");
    let mut raw: Vec<u8> = Vec::new();
    let _ = file
        .read_to_end(&mut raw)
        .expect("Expected to read whole file");

    let mut writer = Cursor::new(Vec::new());
    binrw::BinWrite::write(&p, &mut writer).expect("Expected to write the whole file");

    let inner = writer.into_inner();
    assert_eq!(inner, raw);
}
