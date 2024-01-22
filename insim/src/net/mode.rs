//! Handles encoding and decoding of [Packets](crate::packet::Packet) from the wire.

use bytes::BytesMut;
use std::io;

/// Describes if Insim packets are in "compressed" or "uncompressed" mode.
#[derive(Debug, Clone)]
pub enum Mode {
    /// Insim <= 8 and the LFSWorld relay uses verbatim packet sizes
    Uncompressed,

    /// Insim >= 9 optionally supports "compressing" the packet size by dividing by 4
    /// See <https://www.lfs.net/forum/thread/95662-New-InSim-packet-size-byte-and-mod-info>
    Compressed,
}

impl Mode {
    /// Given a single packet in dst, encode it's length, and ensure that it does not
    /// exceed maximum limits
    #[tracing::instrument]
    pub fn encode_length(&self, len: usize) -> io::Result<u8> {
        if len < self.valid_raw_buffer_min_len() {
            // probably a programming error. lets bail.
            panic!("Failed to encode any data. Possible programming error.");
        }

        // the length passed must include the placeholder byte for the packet size!
        let n = match self {
            Mode::Uncompressed => len,
            Mode::Compressed => {
                if let Some(0) = len.checked_rem(4) {
                    len / 4
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

        Ok(n as u8)
    }

    /// Decode the length of the next packet in the buffer src, ensuring that it does
    /// not exceed limits.
    #[tracing::instrument]
    pub fn decode_length(&self, src: &BytesMut) -> io::Result<Option<usize>> {
        if src.len() < self.valid_raw_buffer_min_len() {
            // Not enough data for even the header
            return Ok(None);
        }

        // get the size of this packet
        let n = match src.first() {
            Some(n) => match self {
                Mode::Uncompressed => *n as usize,
                // if we're in compressed mode, multiply by 4
                Mode::Compressed => (*n as usize) * 4,
            },
            None => return Ok(None),
        };

        // does this exceed the max possible packet?
        if n > self.max_length() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "frame exceeds max_bytes",
            ));
        }

        if src.len() < n {
            // We dont have a full packet yet
            return Ok(None);
        }

        Ok(Some(n))
    }

    fn valid_raw_buffer_min_len(&self) -> usize {
        // All packets are defined as a minimum of:
        // size: u8
        // type: u8
        // reqi: u8
        // data: (at least u8)
        // Assumes we're using a raw buffer
        4
    }

    /// What is the maximum size of a Packet, for a given Mode?
    pub fn max_length(&self) -> usize {
        match self {
            Mode::Uncompressed => 255,
            Mode::Compressed => 1020,
        }
    }
}
