//! Handles encoding and decoding of [Packets](Packet) from the wire.

use crate::{error::Error, packets::Packet, result::Result};
use insim_core::{Decodable, Encodable};
use std::io;
use tokio_util::codec::{Decoder, Encoder};
use tracing;

use bytes::{Buf, BufMut, BytesMut};

#[derive(Debug, Clone, Copy)]
pub enum Mode {
    /// Insim <= 8 uses verbatim packet sizes
    Uncompressed,
    /// Insim >= 9 optionally supports "compressing" the packet size by dividing by 4
    /// See <https://www.lfs.net/forum/thread/95662-New-InSim-packet-size-byte-and-mod-info>
    Compressed,
}

/// A codec for the Insim protocol.
/// This codec handles encoding and decoding of to and from raw bytes to [Packet].
pub struct Codec {
    mode: Mode,
}

impl Codec {
    pub fn new(mode: Mode) -> Codec {
        Codec { mode }
    }

    fn max_possible_length(&self) -> usize {
        match self.mode {
            Mode::Uncompressed => 255,
            Mode::Compressed => 2048,
        }
    }

    fn decode_length(&mut self, src: &mut BytesMut) -> io::Result<Option<usize>> {
        if src.len() < 4 {
            // Not enough data for even the header
            // All packets are defined as a minimum of:
            // size: u8
            // type: u8
            // reqi: u8
            // data: (at least u8)
            return Ok(None);
        }

        let n = {
            let mut src = io::Cursor::new(&mut *src);

            // LFS only communicates in LE (likely it just uses the host native format)
            let n = src.get_u8() as usize;

            // if we're in compressed mode, multiply by 4
            let n = match self.mode {
                Mode::Uncompressed => n,
                Mode::Compressed => n * 4,
            };

            // does this exceed the max possible packet?
            if n > self.max_possible_length() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "frame exceeds max_bytes",
                ));
            }

            // we need to remove the length of the header
            match n.checked_sub(1) {
                Some(n) => n,
                None => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "provided length would overflow after adjustment",
                    ));
                }
            }
        };

        if src.len() < n {
            // We dont have a full packet yet
            return Ok(None);
        }

        // skip over the size field now that we know we have a full packet
        src.advance(1);

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

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>> {
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
                tracing::error!("unhandled error: {:?}, data: {:?}", e, data);
                Err(e.into())
            }
        }
    }
}

impl Encoder<Packet> for Codec {
    type Error = Error;

    fn encode(&mut self, msg: Packet, dst: &mut BytesMut) -> Result<()> {
        let mut buf = BytesMut::new();
        msg.encode(&mut buf, None)?;

        let n = buf.len();

        if n > self.max_possible_length() {
            // probably a programming error. lets bail.
            panic!("Provided length would overflow the maximum byte size of {}. This is probably a programming error, or a change in the packet definition.", self.max_possible_length());
        }

        // Adjust `n` with bounds checking to include the size of the packet
        let n = n.checked_add(1);

        let n = match n {
            Some(n) => n,
            None => {
                // Probably a programming error, lets bail.
                panic!("Provided length would overflow after adjustment. This is probably a programming error.");
            }
        };

        // Reserve capacity in the destination buffer to fit the frame and
        // length field (plus adjustment).
        dst.reserve(n + 1);

        let n = match self.mode {
            Mode::Uncompressed => n,
            Mode::Compressed => {
                if n % 4 == 0 {
                    n / 4
                } else {
                    // probably a programming error, lets bail.
                    panic!("Packet length is not divisible by 4! This is probably a programming error.");
                }
            }
        };

        dst.put_u8(n as u8);

        // Write the frame to the buffer
        dst.extend_from_slice(&buf[..]);

        Ok(())
    }
}
