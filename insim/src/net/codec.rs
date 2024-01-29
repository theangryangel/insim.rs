use super::mode::Mode;

use crate::{packet::Packet, result::Result};
use bytes::{Buf, Bytes, BytesMut};
use insim_core::binrw::{BinRead, BinWrite};
use std::io::{Cursor, Write};

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

    /// Return the currect codec [Mode].
    pub fn mode(&self) -> &Mode {
        &self.mode
    }

    /// Encode a [Packet] into [Bytes].
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

        let mut cursor = std::io::Cursor::new(&data);

        let packet = Packet::read(&mut cursor)?;
        tracing::trace!("Decoded packet={:?}", packet);
        Ok(Some(packet))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{
        identifiers::RequestId,
        insim::{Tiny, TinyType},
        packet::Packet,
    };
    use bytes::BytesMut;
    use tokio_test::assert_ok;

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
