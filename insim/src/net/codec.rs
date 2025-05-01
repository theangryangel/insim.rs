use bytes::{Buf, BufMut, Bytes, BytesMut};
use insim_core::{Decode, Encode};

use super::mode::Mode;
use crate::{packet::Packet, result::Result};

/// Handles the encoding and decoding of Insim packets to and from raw bytes.
/// It automatically handles the encoding of the total size of the packet, and the packet
/// type/identifier.
#[derive(Debug)]
pub struct Codec {
    mode: Mode,
}

impl Codec {
    /// Create a new Codec, with a given [Mode].
    pub fn new(mode: Mode) -> Self {
        Self { mode }
    }

    /// Return the current [Mode].
    pub fn mode(&self) -> &Mode {
        &self.mode
    }

    /// Encode a [Packet] into [Bytes].
    #[tracing::instrument]
    pub fn encode(&self, msg: &Packet) -> Result<Bytes> {
        let mut buf = BytesMut::with_capacity(msg.size_hint());

        // add a placeholder for the size of the packet
        buf.put_u8(0);

        // encode the message
        msg.encode(&mut buf)?;

        let n = self.mode().encode_length(buf.len())?;

        // populate the size
        buf[0] = n;

        tracing::debug!("{:?}", &buf);

        Ok(buf.freeze())
    }

    /// Decode a series of bytes into a [Packet]
    #[tracing::instrument]
    pub fn decode(&self, src: &mut BytesMut) -> Result<Option<Packet>> {
        if src.is_empty() {
            return Ok(None);
        }

        let n = match self.mode().decode_length(src)? {
            Some(n) => n,
            None => {
                return Ok(None);
            },
        };

        let mut data = src.split_to(n);

        // skip over the size field now that we know we have a full packet
        // none of the packet definitions include the size
        data.advance(1);

        let packet = Packet::decode(&mut data.freeze())?;
        tracing::trace!("Decoded packet={:?}", packet);
        Ok(Some(packet))
    }
}

#[cfg(test)]
mod tests {
    use bytes::BytesMut;
    use tokio_test::assert_ok;

    use super::*;
    use crate::{
        identifiers::RequestId,
        insim::{Tiny, TinyType},
        packet::Packet,
    };

    #[tokio::test]
    /// Ensure that Codec can decode a basic small packet
    async fn read_tiny_ping() {
        let mut mock = BytesMut::new();
        mock.extend_from_slice(
            // Packet::Tiny, subtype TinyType::Ping, compressed, reqi=2
            &[1, 3, 2, 3],
        );

        let codec = Codec::new(Mode::Compressed);
        let data = codec.decode(&mut mock);
        assert_ok!(&data);
        let data = data.unwrap();

        assert!(matches!(
            data,
            Some(Packet::Tiny(Tiny {
                reqi: RequestId(2),
                subt: TinyType::Ping,
            }))
        ));
    }

    #[tokio::test]
    /// Ensure that Codec can write a basic small packet
    async fn write_tiny_ping() {
        let mut mock = BytesMut::new();
        mock.extend_from_slice(
            // Packet::Tiny, subtype TinyType::Ping, compressed, reqi=2
            &[1, 3, 2, 3],
        );

        let codec = Codec::new(Mode::Compressed);
        let buf = codec.encode(&Packet::Tiny(Tiny {
            subt: TinyType::Ping,
            reqi: RequestId(2),
        }));
        assert_ok!(&buf);

        assert_eq!(&mock[..], &buf.unwrap()[..])
    }
}
