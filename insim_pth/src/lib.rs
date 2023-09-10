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
//! On an open configuration [Node] are not used and are unavaiable via Insim MCI packets.
//!
//! The distance between each [Node] is not constant. According to the LFS developers
//! there is approximately 0.2 seconds of time between passing one node and the next,
//! when you are "driving at a reasonable speed".

use std::fs;
use std::io::ErrorKind;
use std::io::Read;
use std::path::PathBuf;
use thiserror::Error;

use insim_core::{point::Point, prelude::*, DecodableError};

#[non_exhaustive]
#[derive(Error, Debug)]
pub enum Error {
    #[error("IO Error: {kind}: {message}")]
    IO { kind: ErrorKind, message: String },

    #[error("Failed to decode packet: {0:?}")]
    Decoding(#[from] DecodableError),

    #[error("Remaining bytes after decoding, possibly broken file?")]
    UnexpectedRemainingBytes,
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
#[derive(Debug, InsimDecode, Copy, Clone, Default)]
pub struct Limit {
    pub left: f32,
    pub right: f32,
}

/// Node / or point on a track
#[derive(Debug, InsimDecode, Copy, Clone, Default)]
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

#[derive(Debug, InsimDecode, Default)]
#[insim(magic = b"LFSPTH")]
/// PTH file
pub struct Pth {
    pub version: u8,
    pub revision: u8,

    pub num_nodes: i32,
    pub finish_line_node: i32,

    #[insim(count = "num_nodes")]
    pub nodes: Vec<Node>,
}

impl Pth {
    /// Read and parse a PTH file into a [Pth] struct.
    pub fn from_file(i: &PathBuf) -> Result<Self, Error> {
        if !i.exists() {
            return Err(Error::IO {
                kind: std::io::ErrorKind::NotFound,
                message: format!("Path {i:?} does not exist"),
            });
        }

        let mut input = fs::File::open(i).map_err(Error::from)?;

        let mut buffer = Vec::new();
        input.read_to_end(&mut buffer).map_err(Error::from)?;

        let mut data = insim_core::bytes::BytesMut::new();
        data.extend_from_slice(&buffer);

        let result = Self::decode(&mut data, None)?;

        if data.remaining() > 0 {
            return Err(Error::UnexpectedRemainingBytes);
        }

        Ok(result)
    }
}
