use std::marker::PhantomData;

use bytes::{BytesMut, BufMut, Buf};
use insim_core::{Encodable, Decodable};
use crate::result::Result;

use super::{Mode, Packets};

pub struct Codec<P: Packets> {
    mode: Mode,
    marker: PhantomData<P>
}

impl<P: Packets> Codec<P> {
    pub fn new(mode: Mode) -> Self {
        Self {
            mode,
            marker: PhantomData
        }
    }

    pub fn mode(&self) -> crate::codec::Mode {
        self.mode
    }

    pub fn encode(&self, msg: &P, dst: &mut BytesMut) -> Result<()> {
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

    pub fn decode(&self, src: &mut BytesMut) -> Result<Option<P>> {
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

        let res = P::decode(&mut data, None);

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
