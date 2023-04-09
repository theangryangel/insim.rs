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

impl Mode {
    pub fn encode_length(&self, dst: &mut BytesMut) -> io::Result<usize> {
        // Adjust `n` with bounds checking to include the size of the packet
        let n = match dst.len().checked_add(1) {
            Some(n) => n,
            None => {
                // Probably a programming error, lets bail.
                panic!(
                    "Provided length would overflow after adjustment.
                    This is probably a programming error."
                );
            }
        };

        let n = match self {
            Mode::Uncompressed => n,
            Mode::Compressed => {
                if n % 4 == 0 {
                    n / 4
                } else {
                    // probably a programming error, lets bail.
                    panic!(
                        "Packet length is not divisible by 4!
                        This is probably a programming error."
                    );
                }
            }
        };

        if n > self.max_length() {
            // probably a programming error. lets bail.
            panic!(
                "Provided length would overflow the maximum byte size of {}.
                This is probably a programming error, or a change in the
                packet definition.",
                self.max_length()
            );
        }

        Ok(n)
    }

    pub fn decode_length(&self, src: &mut BytesMut) -> io::Result<Option<usize>> {
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
            // we want a cursor so that we're not fiddling with the internal offset of src
            let mut src = io::Cursor::new(&mut *src);

            // get the size of this packet
            let n = src.get_u8() as usize;

            // if we're in compressed mode, multiply by 4
            let n = match self {
                Mode::Uncompressed => n,
                Mode::Compressed => n * 4,
            };

            // does this exceed the max possible packet?
            if n > self.max_length() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "frame exceeds max_bytes",
                ));
            }

            n
        };

        if src.len() < n {
            // We dont have a full packet yet
            return Ok(None);
        }

        Ok(Some(n))
    }

    pub fn max_length(&self) -> usize {
        match self {
            Mode::Uncompressed => 255,
            Mode::Compressed => 1020,
        }
    }
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

        let n = match self.mode.decode_length(src)? {
            Some(n) => n,
            None => {
                return Ok(None);
            }
        };

        let mut data = src.split_to(n);

        // skip over the size field now that we know we have a full packet
        // none of the packet definitions include the size
        data.advance(1);

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

        let n = self.mode.encode_length(&mut buf)?;

        // Reserve capacity in the destination buffer to fit the frame and
        // length field (plus adjustment).
        dst.reserve(n + 1);

        dst.put_u8(n as u8);

        // Write the frame to the buffer
        dst.extend_from_slice(&buf[..]);

        Ok(())
    }
}
