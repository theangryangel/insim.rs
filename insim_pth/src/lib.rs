//! # insim_pth
//!
//! Parse a Live for Speed pth (path) file.
//!
//! A pth file consists of a series points [Node], with direction and width ([Limit]),
//! that describe the track that you drive along.
//!
//! Historically LFS has used the PTH to watch your progress along the track, decides
//! if you are driving in reverse, the yellow and blue flag systems, the position list,
//! timing, etc.
//!
//! On a standard LFS track the [Node] is communicated via MCI and NLP Insim packets.
//!
//! On an open configuration [Node] are not used and are unavailable via Insim MCI packets.
//!
//! The distance between each [Node] is not constant. According to the LFS developers
//! there is approximately 0.2 seconds of time between passing one node and the next,
//! when you are "driving at a reasonable speed".

#[cfg(test)]
use std::io::{Cursor, Read, Seek, SeekFrom};
use std::{
    fs::{self, File},
    io::ErrorKind,
    path::PathBuf,
};

use binrw::BinRead;
use insim_core::{
    binrw::{self, binrw},
    point::Point,
};
use thiserror::Error;

#[non_exhaustive]
#[derive(Error, Debug)]
#[allow(missing_docs)]
pub enum Error {
    #[error("IO Error: {kind}: {message}")]
    IO { kind: ErrorKind, message: String },

    #[error("BinRw Err {0:?}")]
    BinRwErr(#[from] binrw::Error),
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::IO {
            kind: e.kind(),
            message: e.to_string(),
        }
    }
}

/// Describes the Left and Right limit, of a given node.
#[derive(Debug, Copy, Clone, Default, PartialEq)]
#[binrw]
pub struct Limit {
    /// Left track limit
    pub left: f32,

    /// Right track limit
    pub right: f32,
}

/// Node / or point on a track
#[derive(Debug, Copy, Clone, Default, PartialEq)]
#[binrw]
pub struct Node {
    /// Center point of this node
    pub center: Point<i32>,

    /// Expected direction of travel
    pub direction: Point<f32>,

    /// Track outer limit, relative to the center point and direction of travel
    pub outer_limit: Limit,

    /// Road limit, relative to the center point and direction of travel
    pub road_limit: Limit,
}

impl Node {
    /// Get the center point of this node, optionally scaled
    pub fn get_center(&self, scale: Option<f32>) -> Point<f32> {
        let scale = scale.unwrap_or(1.0);

        Point {
            x: self.center.x as f32 / scale,
            y: self.center.y as f32 / scale,
            z: self.center.z as f32 / scale,
        }
    }

    /// Calculate the absolute position of the left and right road limits
    pub fn get_road_limit(&self, scale: Option<f32>) -> (Point<f32>, Point<f32>) {
        self.calculate_limit_position(&self.road_limit, scale)
    }

    /// Calculate the absolute position of the left and right track limits
    pub fn get_outer_limit(&self, scale: Option<f32>) -> (Point<f32>, Point<f32>) {
        self.calculate_limit_position(&self.outer_limit, scale)
    }

    fn calculate_limit_position(
        &self,
        limit: &Limit,
        scale: Option<f32>,
    ) -> (Point<f32>, Point<f32>) {
        let left_cos = f32::cos(90.0 * std::f32::consts::PI / 180.0);
        let left_sin = f32::sin(90.0 * std::f32::consts::PI / 180.0);
        let right_cos = f32::cos(-90.0 * std::f32::consts::PI / 180.0);
        let right_sin = f32::sin(-90.0 * std::f32::consts::PI / 180.0);

        let center = self.get_center(scale);

        let left: Point<f32> = Point {
            x: ((self.direction.x * left_cos) - (self.direction.y * left_sin)) * limit.left
                + (center.x),
            y: ((self.direction.y * left_cos) + (self.direction.x * left_sin)) * limit.left
                + (center.y),
            z: (center.z),
        };

        let right: Point<f32> = Point {
            x: ((self.direction.x * right_cos) - (self.direction.y * right_sin)) * -limit.right
                + (center.x),
            y: ((self.direction.y * right_cos) + (self.direction.x * right_sin)) * -limit.right
                + (center.y),
            z: (center.z),
        };

        (left, right)
    }
}

#[binrw]
#[brw(little, magic = b"LFSPTH")]
#[derive(Debug, Default, PartialEq)]
/// PTH file
pub struct Pth {
    /// File format version
    pub version: u8,
    /// File format revision
    pub revision: u8,

    #[bw(calc = nodes.len() as i32)]
    num_nodes: i32,

    /// Which node is the finishing line
    pub finish_line_node: i32,

    #[br(count = num_nodes)]
    /// A list of nodes
    pub nodes: Vec<Node>,
}

impl Pth {
    /// Read and parse a PTH file into a [Pth] struct.
    pub fn from_file(i: &mut File) -> Result<Self, Error> {
        Pth::read(i).map_err(Error::from)
    }

    /// Read and parse a PTH file into a [Pth] struct.
    pub fn from_pathbuf(i: &PathBuf) -> Result<Self, Error> {
        if !i.exists() {
            return Err(Error::IO {
                kind: std::io::ErrorKind::NotFound,
                message: format!("Path {i:?} does not exist"),
            });
        }

        let mut input = fs::File::open(i).map_err(Error::from)?;

        Self::from_file(&mut input)
    }
}

#[cfg(test)]
fn assert_valid_as1_pth(p: &Pth) {
    assert_eq!(p.version, 0);
    assert_eq!(p.revision, 0);
    assert_eq!(p.finish_line_node, 250);
}

#[test]
fn test_pth_decode_from_pathbuf() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("./tests/AS1.pth");
    let p = Pth::from_pathbuf(&path).expect("Expected PTH file to be parsed");

    assert_valid_as1_pth(&p)
}

#[test]
fn test_pth_decode_from_file() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("./tests/AS1.pth");
    let mut file = File::open(path).expect("Expected Autocross_3DH.smx to exist");
    let p = Pth::from_file(&mut file).expect("Expected PTH file to be parsed");

    let pos = file.stream_position().unwrap();
    let end = file.seek(SeekFrom::End(0)).unwrap();

    assert_eq!(pos, end, "Expected the whole file to be completely read");

    assert_valid_as1_pth(&p)
}

#[test]
fn test_pth_encode() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("./tests/AS1.pth");
    let p = Pth::from_pathbuf(&path).expect("Expected SMX file to be parsed");

    let mut file = File::open(path).expect("Expected AS1.pth to exist");
    let mut raw: Vec<u8> = Vec::new();
    let _ = file
        .read_to_end(&mut raw)
        .expect("Expected to read whole file");

    let mut writer = Cursor::new(Vec::new());
    binrw::BinWrite::write(&p, &mut writer).expect("Expected to write the whole file");

    let inner = writer.into_inner();
    assert_eq!(inner, raw);
}
