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
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::IO {
            kind: e.kind(),
            message: e.to_string(),
        }
    }
}

#[derive(Debug, InsimDecode, Copy, Clone, Default)]
pub struct Limit {
    pub left: f32,
    pub right: f32,
}

#[derive(Debug, InsimDecode, Copy, Clone, Default)]
pub struct Node {
    pub center: Point<i32>,
    pub direction: Point<f32>,

    pub outer_limit: Limit,
    pub road_limit: Limit,
}

impl Node {
    pub fn get_center(&self, scale: Option<f32>) -> Point<f32> {
        let scale = scale.unwrap_or(1.0);

        Point {
            x: self.center.x as f32 / scale,
            y: self.center.y as f32 / scale,
            z: self.center.z as f32 / scale,
        }
    }

    pub fn get_road_limit(&self, scale: Option<f32>) -> (Point<f32>, Point<f32>) {
        self.calculate_limit_position(&self.road_limit, scale)
    }

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
pub struct Pth {
    pub version: u8,
    pub revision: u8,

    pub num_nodes: i32,
    pub finish_line_node: i32,

    #[insim(count = "num_nodes")]
    pub nodes: Vec<Node>,
}

impl Pth {
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

        Ok(Self::decode(&mut data, None)?)
    }
}
