//! Pin file

use std::{
    fs,
    io::{Read, Write},
    path::Path,
};

use bytes::{Bytes, BytesMut};
use insim_core::{Decode, Encode};

#[derive(Debug, PartialEq)]
/// PIN file
pub enum Pin {
    /// LFSPIN file, version 0, revision 0
    LfsPin0(super::lfspin::v0::LfsPin),
}

impl Pin {
    // XXX: Most pin files are only around 32 bytes, so we're going to pre-allocate at least that
    // amount of space.
    const DEFAULT_CAPACITY: usize = 32;

    /// Read and parse a PIN file into a [Pin] struct.
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
            (b"LFSPIN", 0, 0) => {
                let mut data = Vec::with_capacity(Self::DEFAULT_CAPACITY);
                data.push(0);
                let _ = reader.read_to_end(&mut data)?;
                let mut buf = Bytes::from(data);

                Ok(Self::LfsPin0(super::lfspin::v0::LfsPin::decode(&mut buf)?))
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
            Pin::LfsPin0(inner) => {
                inner.encode(&mut buf)?;
                written += writer.write(b"LFSPIN\0")?;
                written += writer.write(&buf[..])?;
            },
        }

        Ok(written)
    }

    /// Read and parse a PIN file into a [Pin] struct.
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
}
