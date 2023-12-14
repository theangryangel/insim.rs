mod mode;

#[cfg(test)]
mod tests;

pub use mode::Mode;

use crate::{packet::Packet, result::Result};
use bytes::{Buf, Bytes, BytesMut};
use insim_core::binrw::{BinRead, BinWrite};
use std::io::{Cursor, Write};

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
        // encode the message
        let mut writer = Cursor::new(Vec::with_capacity(msg.size_hint()));
        let _ = writer.write(&[0]);
        msg.write(&mut writer)?;

        let pos = writer.position();

        // encode the length of the packet, including the placeholder for the length
        let n = self.mode().encode_length(pos as usize)?;

        // update the length the encoded length
        writer.set_position(0);
        writer.write_all(&[n])?;

        let data = writer.into_inner();
        tracing::debug!("{:?}", &data);

        Ok(data.into())
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

        let packet = Packet::read(&mut cursor)?;
        tracing::trace!("Decoded packet={:?}", packet);
        Ok(Some(packet))
    }
}
