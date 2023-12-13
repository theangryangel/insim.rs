mod mode;

#[cfg(test)]
mod tests;

pub use mode::Mode;

use crate::{packet::Packet, result::Result};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use insim_core::binrw::{BinRead, BinWrite};
use std::io::Cursor;

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
        let mut writer = Cursor::new(Vec::new());
        msg.write(&mut writer).unwrap();
        dst.extend_from_slice(&writer.into_inner());

        // encode the length of the packet, including the placeholder for the length
        let n = self.mode().encode_length(dst.len())?;

        // update the length the encoded length
        dst[0] = n;

        tracing::debug!("{:?}", dst);

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

        let mut cursor = std::io::Cursor::new(&data);

        let res = Packet::read(&mut cursor);

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
