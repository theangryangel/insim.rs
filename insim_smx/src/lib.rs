#![deny(unused_crate_dependencies)]

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

use insim_core::binrw::{self, binrw, BinRead};
use insim_core::string::{binrw_parse_codepage_string, binrw_write_codepage_string};
use std::fs::{self, File};
use std::io::ErrorKind;
use std::path::PathBuf;
use thiserror::Error;

use insim_core::point::Point;

#[non_exhaustive]
#[derive(Error, Debug)]
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
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[binrw]
#[derive(Debug, Default, Clone)]
pub struct Argb {
    pub a: u8,
    pub rgb: Rgb,
}

#[binrw]
#[derive(Debug, Default, Clone)]
pub struct ObjectPoint {
    pub xyz: Point<i32>,
    pub colour: Argb,
}

#[binrw]
#[derive(Debug, Default, Clone)]
pub struct Triangle {
    pub a: u16, // index of the objectpoint
    pub b: u16,
    #[brw(pad_after = 2)]
    pub c: u16,
}

#[binrw]
#[derive(Debug, Default, Clone)]
pub struct Object {
    pub center: Point<i32>,
    pub radius: i32,

    #[bw(calc = points.len() as i32)]
    num_object_points: i32,

    #[bw(calc = triangles.len() as i32)]
    num_triangles: i32,

    #[br(count = num_object_points)]
    pub points: Vec<ObjectPoint>,

    #[br(count = num_triangles)]
    pub triangles: Vec<Triangle>,
}

#[binrw]
#[derive(Debug, Default, Clone)]
#[brw(magic = b"LFSSMX", little)]
/// Smx file
pub struct Smx {
    pub game_version: u8,
    pub game_revision: u8,

    pub smx_version: u8,

    pub dimensions: u8,
    pub resolution: u8,

    #[brw(pad_after = 4)]
    pub vertex_colours: u8,

    #[bw(write_with = binrw_write_codepage_string::<32, _>)]
    #[br(parse_with = binrw_parse_codepage_string::<32, _>)]
    pub track: String,

    #[brw(pad_after = 9)]
    pub ground_colour: Rgb,

    #[bw(calc = objects.len() as i32)]
    num_objects: i32,

    #[br(count = num_objects)]
    pub objects: Vec<Object>,

    #[bw(calc = checkpoint_object_index.len() as i32)]
    num_checkpoints: i32,

    #[br(count = num_checkpoints)]
    pub checkpoint_object_index: Vec<i32>,
}

impl Smx {
    /// Read and parse a PTH file into a [Pth] struct.
    pub fn from_file(i: &mut File) -> Result<Self, Error> {
        Self::read(i).map_err(Error::from).map_err(Error::from)
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
