use std::time::{Duration, Instant};

use bytes::{Buf, BufMut, Bytes, BytesMut};
use insim_core::{Decode, Encode};

use super::{mode::Mode, DEFAULT_TIMEOUT_SECS};
use crate::{
    identifiers::RequestId,
    insim::{Tiny, TinyType},
    packet::Packet,
    result::Result,
    Error, WithRequestId,
};

/// Handles the encoding and decoding of Insim packets to and from raw bytes.
/// It automatically handles the encoding of the total size of the packet, and the packet
/// type/identifier.
#[derive(Debug)]
pub struct Codec {
    mode: Mode,
    timeout_at: Instant,
    keepalive: bool,
}

impl Codec {
    /// Create a new Codec, with a given [Mode].
    pub fn new(mode: Mode) -> Self {
        Self {
            mode,
            timeout_at: Instant::now() + Duration::from_secs(DEFAULT_TIMEOUT_SECS),
            keepalive: false,
        }
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
    pub fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Packet>> {
        if src.is_empty() {
            return Ok(None);
        }

        let n = match self.mode().decode_length(src)? {
            Some(n) => n,
            None => {
                return Ok(None);
            },
        };

        let mut data = src.split_to(n).freeze();

        self.timeout_at = Instant::now() + Duration::from_secs(DEFAULT_TIMEOUT_SECS);

        // cloning Bytes is cheap:
        // Bytes values facilitate zero-copy network programming by allowing multiple Bytes objects to point to the same underlying memory.
        let original = data.clone();

        // skip over the size field now that we know we have a full packet
        // none of the packet definitions include the size
        data.advance(1);

        let packet = Packet::decode(&mut data);
        match packet {
            Ok(packet) => {
                tracing::trace!("Decoded packet={:?}", packet);
                if data.remaining() > 0 {
                    return Err(Error::IncompleteDecode {
                        input: original,
                        decoded: packet,
                        remaining: data,
                    });
                }

                if matches!(
                    packet,
                    Packet::Tiny(Tiny {
                        subt: TinyType::None,
                        reqi: RequestId(0)
                    })
                ) {
                    self.keepalive = true;
                }

                Ok(Some(packet))
            },
            Err(e) => Err(crate::Error::Decode {
                offset: data.as_ptr() as usize - original.as_ptr() as usize,
                error: e,
                input: original,
            }),
        }
    }

    /// Return the next timeout
    pub fn timeout(&self) -> Instant {
        self.timeout_at
    }

    /// Has the connection timedout?
    pub fn reached_timeout(&self) -> bool {
        self.timeout_at <= Instant::now()
    }

    /// If Some value is returned, this needs to be sent to maintain the connection
    pub fn keepalive(&mut self) -> Option<Packet> {
        if self.keepalive {
            self.keepalive = false;
            Some(TinyType::None.with_request_id(RequestId(0)).into())
        } else {
            None
        }
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

        let mut codec = Codec::new(Mode::Compressed);
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
