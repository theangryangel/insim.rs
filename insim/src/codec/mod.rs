mod mode;

#[cfg(test)]
mod tests;

pub use mode::Mode;

use crate::{packet::Packet, result::Result};
use bytes::{Buf, BufMut, BytesMut};
use insim_core::{Decodable, Encodable};

pub struct Codec {
    mode: Mode,
}

impl Codec {
    pub fn new(mode: Mode) -> Self {
        Self { mode }
    }

    pub fn mode(&self) -> crate::codec::Mode {
        self.mode
    }

    pub fn encode(&self, msg: &Packet, dst: &mut BytesMut) -> Result<()> {
        let mut buf = BytesMut::new();
        msg.encode(&mut buf, None)?;

        let n = self.mode().encode_length(&mut buf)?;

        // Reserve capacity in the destination buffer to fit the frame and
        // length field (plus adjustment).
        dst.reserve(n + 1);

        dst.put_u8(n as u8);

        // Write the frame to the buffer
        dst.extend_from_slice(&buf[..]);

        Ok(())
    }

    pub fn decode(&self, src: &mut BytesMut) -> Result<Option<Packet>> {
        if src.is_empty() {
            return Ok(None);
        }

        let n = match self.mode().decode_length(src)? {
            Some(n) => n,
            None => {
                return Ok(None);
            }
        };

        let mut data = src.split_to(n);

        // skip over the size field now that we know we have a full packet
        // none of the packet definitions include the size
        data.advance(1);

        let res = Packet::decode(&mut data, None);

        match res {
            Ok(packet) => {
                tracing::trace!("Decoded packet={:?}", packet);
                Ok(Some(packet))
            }
            Err(e) => {
                tracing::error!("Unhandled error decoding packet: {:?}, data={:?}", e, data);
                Err(e.into())
            }
        }
    }
}
