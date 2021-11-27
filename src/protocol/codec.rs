use super::Packet;
use deku::{DekuContainerWrite, DekuError};
use std::convert::TryFrom;
use std::io;
use tokio_util::codec::{Decoder, Encoder};
use tracing;

use bytes::{Buf, BufMut, Bytes, BytesMut};

pub enum InsimCodecMode {
    // Insim <= 8 uses verbatim packet sizes
    Verbatim,
    // Insim >= 9 uses the size divided by 4
    // https://www.lfs.net/forum/thread/95662-New-InSim-packet-size-byte-and-mod-info
    DivFour,
}

pub struct InsimCodec {
    mode: InsimCodecMode,
    max_bytes: usize,
    length_bytes: usize,
}

impl InsimCodec {
    pub fn new() -> InsimCodec {
        InsimCodec {
            mode: InsimCodecMode::Verbatim,
            max_bytes: 1_024 * 1_024,
            length_bytes: 1,
        }
    }

    fn decode_length(&mut self, src: &mut BytesMut) -> io::Result<Option<usize>> {
        if src.len() < self.length_bytes {
            // Not enough data for even the header
            return Ok(None);
        }

        let n = {
            let mut src = io::Cursor::new(&mut *src);

            // LFS only communicates in LE (likely it just uses the host native format)
            let n = src.get_uint_le(self.length_bytes);

            if n > self.max_bytes as u64 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "frame exceeds max_bytes",
                ));
            }

            // The check above ensures there is no overflow
            let n = n as usize;

            // we need to remove the length of the header
            let n = n.checked_sub(self.length_bytes);

            match n {
                Some(n) => n,
                None => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "provided length would overflow after adjustment",
                    ));
                }
            }
        };

        let n = match self.mode {
            InsimCodecMode::Verbatim => n,
            InsimCodecMode::DivFour => n * 4,
        };

        if (src.len() - self.length_bytes) < n {
            // We dont have a full packet yet
            return Ok(None);
        }

        // skip over the length field now that we know we have a full packet
        src.advance(self.length_bytes);

        Ok(Some(n))
    }
}

impl Default for InsimCodec {
    fn default() -> Self {
        Self::new()
    }
}

impl Decoder for InsimCodec {
    type Item = Packet;

    // TODO return custom error
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> io::Result<Option<Self::Item>> {
        if src.is_empty() {
            return Ok(None);
        }

        let n = match self.decode_length(src)? {
            Some(n) => n,
            None => {
                return Ok(None);
            }
        };

        let data = src.split_to(n);

        let res = Self::Item::try_from(data.as_ref());

        match res {
            Ok(packet) => Ok(Some(packet)),
            Err(DekuError::Incomplete(e)) => {
                // If we're here, everything is going wonky.
                tracing::error!(
                    "malformed packet! this is probably a programming error, error: {:?}, input: {:?}",
                    e,
                    data.to_vec(),
                );
                Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "incomplete or malformed packet",
                ))
            }
            Err(DekuError::Parse(e)) => {
                tracing::warn!("unsupported packet: {:?}", e);
                Ok(None)
            }
            Err(e) => Err(io::Error::new(io::ErrorKind::InvalidInput, e.to_string())),
        }
    }
}

impl Encoder<Packet> for InsimCodec {
    type Error = io::Error;

    fn encode(&mut self, msg: Packet, dst: &mut BytesMut) -> Result<(), io::Error> {
        let data = Bytes::from(msg.to_bytes().unwrap());

        let n = data.len();

        if n > self.max_bytes {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "frame exceeds max_bytes",
            ));
        }

        // Adjust `n` with bounds checking
        let n = n.checked_add(self.length_bytes as usize);

        let n = match n {
            Some(n) => n,
            None => {
                return Err(Self::Error::new(
                    io::ErrorKind::InvalidInput,
                    "provided length would overflow after adjustment",
                ));
            }
        };

        // Reserve capacity in the destination buffer to fit the frame and
        // length field (plus adjustment).
        dst.reserve(self.length_bytes + n);

        let n = match self.mode {
            InsimCodecMode::Verbatim => n,
            InsimCodecMode::DivFour => n / 4,
        };

        dst.put_uint_le(n as u64, self.length_bytes);

        // Write the frame to the buffer
        dst.extend_from_slice(&data[..]);

        Ok(())
    }
}
