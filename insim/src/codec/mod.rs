mod mode;

#[cfg(test)]
mod tests;

pub use mode::Mode;

use crate::{packet::Packet, result::Result};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use insim_core::{Decodable, Encodable};

#[derive(Debug)]
pub struct Codec {
    mode: Mode,
}

impl Codec {
    pub fn new(mode: Mode) -> Self {
        Self { mode }
    }

    pub fn mode(&self) -> &Mode {
        &self.mode
    }

    #[tracing::instrument]
    pub fn encode(&self, msg: &Packet) -> Result<Bytes> {
        let mut dst = BytesMut::new();

        // put a placeholder for our length, we'll come back to this later
        dst.put_u8(0);

        // encode the message
        msg.encode(&mut dst, None)?;

        // encode the length of the packet, including the placeholder for the length
        let n = self.mode().encode_length(dst.len())?;

        // update the length the encoded length
        dst[0] = n;

        Ok(dst.freeze())
    }

    #[tracing::instrument]
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
