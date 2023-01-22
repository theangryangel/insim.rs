//! Handles encoding and decoding of [Packets](Packet) from the wire.

use super::Packet;
use crate::error::Error;
use std::convert::TryFrom;
use std::io;
use tokio_util::codec::{Decoder, Encoder};
use tracing;
use insim_core::{InsimDecodable, InsimEncodable};

use bytes::{Buf, BufMut, Bytes, BytesMut};

#[derive(Clone, Copy)]
pub enum Mode {
    /// Insim <= 8 uses verbatim packet sizes
    Uncompressed,
    /// Insim >= 9 optionally supports "compressing" the packet size by dividing by 4
    /// See https://www.lfs.net/forum/thread/95662-New-InSim-packet-size-byte-and-mod-info
    Compressed,
}

/// A codec for the Insim protocol.
/// This codec handles encoding and decoding of to and from raw bytes to [Packet].
pub struct Codec {
    mode: Mode,
    max_bytes: usize,
    length_bytes: usize,
}

impl Codec {
    pub fn new(mode: Mode) -> Codec {
        Codec {
            mode,
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
            Mode::Uncompressed => n,
            Mode::Compressed => n * 4,
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

impl Default for Codec {
    fn default() -> Self {
        Self::new(Mode::Uncompressed)
    }
}

impl Decoder for Codec {
    type Item = Packet;
    type Error = Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.is_empty() {
            return Ok(None);
        }

        let n = match self.decode_length(src)? {
            Some(n) => n,
            None => {
                return Ok(None);
            }
        };

        let mut data = src.split_to(n);

        let res = Self::Item::decode(&mut data, None);

        match res {
            Ok(packet) => {
                tracing::debug!("decoded: {:?}", packet);
                Ok(Some(packet))
            }
            Err(e) => {
                tracing::error!("unhandled error: {:?}", e);
                panic!("unhandled error: {:?}", e);
                Err(e.into())
            }
        }
    }
}

impl Encoder<Packet> for Codec {
    type Error = Error;

    fn encode(&mut self, msg: Packet, dst: &mut BytesMut) -> Result<(), Error> {
        let data = msg.to_bytes()?;
        let data = msg.encode(buf)?;

        let n = data.len();

        if n > self.max_bytes {
            return Err(Error::IO {
                kind: io::ErrorKind::InvalidInput,
                message: "frame exceeds max_bytes".into(),
            });
        }

        // Adjust `n` with bounds checking
        let n = n.checked_add(self.length_bytes);

        let n = match n {
            Some(n) => n,
            None => {
                // Probably a programming error, lets bail.
                panic!("provided length would overflow after adjustment");
            }
        };

        // Reserve capacity in the destination buffer to fit the frame and
        // length field (plus adjustment).
        dst.reserve(self.length_bytes + n);

        let n = match self.mode {
            Mode::Uncompressed => n,
            Mode::Compressed => {
                if n % 4 == 0 {
                    n / 4
                } else {
                    return Err(Error::IO {
                        kind: io::ErrorKind::InvalidInput,
                        message: "frame is not divisible by 4".into(),
                    });
                }
            }
        };

        dst.put_uint_le(n as u64, self.length_bytes);

        // Write the frame to the buffer
        dst.extend_from_slice(&data[..]);

        Ok(())
    }
}
