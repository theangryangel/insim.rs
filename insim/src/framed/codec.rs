use std::fmt::Debug;

use bytes::{BytesMut, Bytes, BufMut, Buf};
use if_chain::if_chain;
use insim_core::{Encodable, Decodable};
use crate::{error::Error, result::Result};

pub trait Codec {
    type Item: Encodable + Decodable + Debug;
    const VERSION: u8 = 0;

    fn mode(&self) -> crate::codec::Mode;

    fn encode(&self, msg: &Self::Item, dst: &mut BytesMut) -> Result<()> {
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

    fn decode(&self, src: &mut BytesMut) -> Result<Option<Self::Item>> {
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

pub mod v9 {

    pub use crate::packets::Packet;
    pub use crate::packets::{insim, relay};

    pub struct Codec {
        pub mode: crate::codec::Mode,
    }

    impl super::Codec for Codec {
        type Item = Packet;
        const VERSION: u8 = 9;

        fn mode(&self) -> crate::codec::Mode {
            self.mode
        }
    }
}
