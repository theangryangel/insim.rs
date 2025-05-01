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

use std::{
    fs::{self, File},
    io::{ErrorKind, Read},
    path::PathBuf,
};

use bytes::{Buf, BufMut, Bytes, BytesMut};
use insim_core::{point::Point, Ascii, Codepage, Decode, Encode};
use thiserror::Error;

#[non_exhaustive]
#[derive(Error, Debug)]
#[allow(missing_docs)]
pub enum Error {
    #[error("IO Error: {kind}: {message}")]
    IO { kind: ErrorKind, message: String },

    #[error("BinRw Err {0:?}")]
    ReadWriteBuf(#[from] insim_core::Error),
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::IO {
            kind: e.kind(),
            message: e.to_string(),
        }
    }
}

#[derive(Debug, Default, Clone, insim_macros::ReadWriteBuf)]
/// Red Green Blue
pub struct Rgb {
    /// Red
    pub r: u8,
    /// Green
    pub g: u8,
    /// Blue
    pub b: u8,
}

#[derive(Debug, Default, Clone, insim_macros::ReadWriteBuf)]
/// RGB with alpha channel
pub struct Argb {
    /// Alpha
    pub a: u8,
    /// RGB
    pub rgb: Rgb,
}

#[derive(Debug, Default, Clone, insim_macros::ReadWriteBuf)]
/// An Object at a given point with a colour
pub struct ObjectPoint {
    /// Position/point
    pub xyz: Point<i32>,
    /// Colour
    pub colour: Argb,
}

#[derive(Debug, Default, Clone, insim_macros::ReadWriteBuf)]
/// Triangle block
pub struct Triangle {
    /// Vertex A
    pub a: u16,
    /// Vertex B
    pub b: u16,
    #[read_write_buf(pad_after = 2)]
    /// Vertex C
    pub c: u16,
}

#[derive(Debug, Default, Clone)]
/// Object Block
pub struct Object {
    /// Center point of object
    pub center: Point<i32>,
    /// Radius of object
    pub radius: i32,

    /// List of points
    pub points: Vec<ObjectPoint>,

    /// list of triangles
    pub triangles: Vec<Triangle>,
}

impl Decode for Object {
    fn decode(buf: &mut Bytes) -> Result<Self, insim_core::Error> {
        let center = Point::<i32>::decode(buf)?;
        let radius = i32::decode(buf)?;
        let mut num_object_points = i32::decode(buf)?;
        let mut num_triangles = i32::decode(buf)?;
        let mut points = Vec::new();
        let mut triangles = Vec::new();
        while num_object_points > 0 {
            points.push(ObjectPoint::decode(buf)?);
            num_object_points -= 1;
        }
        while num_triangles > 0 {
            triangles.push(Triangle::decode(buf)?);
            num_triangles -= 1;
        }
        Ok(Self {
            center,
            radius,
            points,
            triangles,
        })
    }
}

impl Encode for Object {
    fn encode(&self, buf: &mut BytesMut) -> Result<(), insim_core::Error> {
        self.center.encode(buf)?;
        self.radius.encode(buf)?;
        (self.points.len() as i32).encode(buf)?;
        (self.triangles.len() as i32).encode(buf)?;
        for i in self.points.iter() {
            i.encode(buf)?;
        }
        for i in self.triangles.iter() {
            i.encode(buf)?;
        }
        Ok(())
    }
}

#[derive(Debug, Default, Clone)]
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

    /// Always 1
    pub vertex_colours: u8,

    /// Track
    pub track: String,

    /// Colour of ground
    pub ground_colour: Rgb,

    /// List of objects
    pub objects: Vec<Object>,

    /// List of checkpoints
    pub checkpoint_object_index: Vec<i32>,
}

impl Decode for Smx {
    fn decode(buf: &mut Bytes) -> Result<Self, insim_core::Error> {
        let magic = String::from_ascii_bytes(buf, 6)?;
        if magic != "LFSSMX" {
            unimplemented!("Not a LFS SMX file");
        }
        let game_version = u8::decode(buf)?;
        let game_revision = u8::decode(buf)?;
        let smx_version = u8::decode(buf)?;
        let dimensions = u8::decode(buf)?;
        let resolution = u8::decode(buf)?;
        let vertex_colours = u8::decode(buf)?;
        buf.advance(4);
        let track = String::from_codepage_bytes(buf, 32)?;
        let ground_colour = Rgb::decode(buf)?;
        buf.advance(9);
        let mut num_objects = i32::decode(buf)?;
        let mut objects = Vec::new();
        while num_objects > 0 {
            objects.push(Object::decode(buf)?);
            num_objects -= 1;
        }
        let mut num_checkpoints = i32::decode(buf)?;
        let mut checkpoint_object_index = Vec::new();
        while num_checkpoints > 0 {
            checkpoint_object_index.push(i32::decode(buf)?);
            num_checkpoints -= 1;
        }
        Ok(Self {
            game_version,
            game_revision,
            smx_version,
            dimensions,
            resolution,
            vertex_colours,
            track,
            ground_colour,
            objects,
            checkpoint_object_index,
        })
    }
}

impl Encode for Smx {
    fn encode(&self, buf: &mut BytesMut) -> Result<(), insim_core::Error> {
        buf.extend_from_slice(b"LFSSMX");
        self.game_version.encode(buf)?;
        self.game_revision.encode(buf)?;
        self.smx_version.encode(buf)?;
        self.dimensions.encode(buf)?;
        self.resolution.encode(buf)?;
        self.vertex_colours.encode(buf)?;
        buf.put_bytes(0, 4);
        self.track.to_codepage_bytes(buf, 32, false)?;
        self.ground_colour.encode(buf)?;
        buf.put_bytes(0, 9);
        (self.objects.len() as i32).encode(buf)?;
        for i in self.objects.iter() {
            i.encode(buf)?;
        }
        (self.checkpoint_object_index.len() as i32).encode(buf)?;
        for i in self.checkpoint_object_index.iter() {
            i.encode(buf)?;
        }
        Ok(())
    }
}

impl Smx {
    /// Read and parse a SMX file into a [Smx] struct.
    pub fn from_file(i: &mut File) -> Result<Self, Error> {
        let mut data = Vec::new();
        let _ = i.read_to_end(&mut data)?;
        let mut data = Bytes::from(data);
        Self::decode(&mut data).map_err(Error::from)
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
        Self::from_file(&mut input)
    }
}

#[cfg(test)]
mod test {
    use super::*;

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

        let mut inner = BytesMut::new();
        p.encode(&mut inner).expect("Should not fail to write SMX");

        assert_eq!(inner.len(), raw.len());
        assert_eq!(inner.as_ref(), raw);
    }
}
