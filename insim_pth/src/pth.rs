//! Pth file

use std::{
    fs,
    io::{Read, Write},
    path::Path,
};

use bytes::{Bytes, BytesMut};
use insim_core::{Decode, Encode};

use crate::node;

#[derive(Debug, PartialEq)]
/// PTH file
pub enum Pth {
    /// LFSPTH file, version 0, revision 0
    LfsPth0(super::lfspth::v0::LfsPth),
    /// SRPATH version 0, revision <= 252
    SrPath0(super::srpath::v0::SrPth),
}

impl Pth {
    // XXX: Most pth files are only around 24KB, so we're going to pre-allocate at least that
    // amount of space.
    const DEFAULT_CAPACITY: usize = 24 * 1024;

    /// Read and parse a PTH file into a [Pth] struct.
    pub fn read<R: Read>(mut reader: R) -> Result<Self, super::Error> {
        // Read the common header, MAGIC + version + revision
        let mut header = [0u8; 8];
        reader.read_exact(&mut header)?;
        let magic = &header[0..6];
        let version = header[6];
        let revision = header[7];

        // XXX: Reading into memory using read_to_end should be fine for the small files we're working with
        // here and we avoid any memory mapping, etc.

        match (magic, version, revision) {
            (b"LFSPTH", 0, 0) => {
                let mut data = Vec::with_capacity(Self::DEFAULT_CAPACITY);
                data.push(0);
                let _ = reader.read_to_end(&mut data)?;
                let mut buf = Bytes::from(data);

                Ok(Self::LfsPth0(super::lfspth::v0::LfsPth::decode(&mut buf)?))
            },

            (b"SRPATH", 0, r) if r <= 252 => {
                let mut data = Vec::with_capacity(Self::DEFAULT_CAPACITY);
                data.push(revision);
                let _ = reader.read_to_end(&mut data)?;
                let mut buf = Bytes::from(data);
                Ok(Self::SrPath0(super::srpath::v0::SrPth::decode(&mut buf)?))
            },
            _ => Err(super::Error::UnsupportedVersion {
                magic: magic.to_vec(),
                version,
                revision,
            }),
        }
    }

    /// Write a file
    pub fn write<W: Write>(&self, mut writer: W) -> Result<usize, super::Error> {
        let mut written: usize = 0;
        let mut buf = BytesMut::new();
        match self {
            Pth::LfsPth0(inner) => {
                inner.encode(&mut buf)?;
                written += writer.write(b"LFSPTH\0")?;
                written += writer.write(&buf[..])?;
            },
            Pth::SrPath0(sr_pthv0r252) => {
                sr_pthv0r252.encode(&mut buf)?;
                written += writer.write(b"SRPATH\0")?;
                written += writer.write(&buf[..])?;
            },
        }

        Ok(written)
    }

    /// Read and parse a PTH file into a [Pth] struct.
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, super::Error> {
        let path = path.as_ref();
        if !path.exists() {
            return Err(super::Error::IO {
                kind: std::io::ErrorKind::NotFound,
                message: format!("Path {path:?} does not exist"),
            });
        }

        let mut input = fs::File::open(path).map_err(super::Error::from)?;
        Self::read(&mut input)
    }

    /// iter all nodes
    pub fn iter_nodes(&self) -> Box<dyn Iterator<Item = &node::Node> + '_> {
        match self {
            Pth::LfsPth0(lfs_pth) => Box::new(lfs_pth.nodes.iter()),
            Pth::SrPath0(sr_pth) => {
                // Note: we use .node or the deref.
                // Sn.deref() returns &Node, so sn.deref() is what we want.
                Box::new(sr_pth.main_nodes.iter().map(|sn| &sn.node))
            },
        }
    }
}
