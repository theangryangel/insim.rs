use std::{
    io,
    time::{Duration, Instant},
};

use bytes::{Buf, BufMut, Bytes, BytesMut};
use insim_core::{Decode, Encode};

use super::DEFAULT_TIMEOUT_SECS;
use crate::{
    DEFAULT_BUFFER_CAPACITY, Error, VERSION, WithRequestId,
    identifiers::RequestId,
    insim::{Tiny, TinyType, Ver},
    packet::Packet,
    result::Result,
};

const MAX_PACKET_SIZE: usize = 1020;
const MIN_PACKET_SIZE: usize = 4;

/// Handles the encoding and decoding of Insim packets to and from raw bytes.
/// It automatically handles the encoding of the total size of the packet, and the packet
/// type/identifier.
/// You may use this with your own IO, either sync or async. See Framed as an example of doing so.
/// This is not responsible for managing the initial handshake, nor will it automatically send any
/// keepalives. But it will indicate if a keepalive should be sent and when an IO timeout should
/// occur.
/// With your own IO it is your responsibility to ensure that your read and write timeouts under
/// blocking implementations are appropriately set. We recommend
/// [crate::net::DEFAULT_TIMEOUT_SECS].
#[derive(Debug)]
pub struct Codec {
    timeout_at: Instant,
    keepalive: bool,
    buffer: BytesMut,
}

impl Default for Codec {
    fn default() -> Self {
        Self::new()
    }
}

impl Codec {
    /// Create a new Codec
    pub fn new() -> Self {
        Self {
            timeout_at: Instant::now() + Duration::from_secs(DEFAULT_TIMEOUT_SECS),
            keepalive: false,
            buffer: BytesMut::with_capacity(DEFAULT_BUFFER_CAPACITY),
        }
    }

    /// Encode a [Packet] into [Bytes].
    #[tracing::instrument]
    pub fn encode(&self, msg: &Packet) -> Result<Bytes> {
        let mut buf = BytesMut::with_capacity(msg.size_hint());

        // add a placeholder for the size of the packet
        buf.put_u8(0);

        // encode the message
        msg.encode(&mut buf)?;

        let n = self.encode_length(buf.len())?;

        // populate the size
        buf[0] = n;

        tracing::debug!("{:?}", &buf);

        Ok(buf.freeze())
    }

    /// Feed
    #[tracing::instrument]
    pub fn feed(&mut self, src: &[u8]) {
        if src.is_empty() {
            return;
        }

        self.buffer.extend_from_slice(src);
    }

    /// Decode any complete packet in the buffer into a [Packet]
    #[tracing::instrument]
    pub fn decode(&mut self) -> Result<Option<Packet>> {
        if self.buffer.is_empty() {
            return Ok(None);
        }

        let n = match self.decode_length(&self.buffer)? {
            Some(n) => n,
            None => {
                return Ok(None);
            },
        };

        let mut data = self.buffer.split_to(n).freeze();

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
                tracing::debug!("{:?}", packet);
                if data.remaining() > 0 {
                    return Err(Error::IncompleteDecode {
                        input: original,
                        remaining: data,
                    });
                }

                match packet {
                    Packet::Tiny(Tiny {
                        subt: TinyType::None,
                        reqi: RequestId(0),
                    }) => {
                        self.keepalive = true;
                    },
                    Packet::Ver(Ver { insimver, .. }) => {
                        if insimver != VERSION {
                            return Err(Error::IncompatibleVersion(insimver));
                        }
                    },
                    _ => {},
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

    /// Given a single packet in dst, encode it's length, and ensure that it does not
    /// exceed maximum limits
    #[tracing::instrument]
    fn encode_length(&self, len: usize) -> io::Result<u8> {
        if len < MIN_PACKET_SIZE {
            // probably a programming error. lets bail.
            panic!("Failed to encode any data. Possible programming error.");
        }

        let n = if let Some(0) = len.checked_rem(4) {
            len / 4
        } else {
            // probably a programming error, lets bail.
            panic!(
                "Packet length is not divisible by 4!
                This is probably a programming error."
            );
        };

        if n > MAX_PACKET_SIZE {
            // probably a programming error. lets bail.
            panic!(
                "Provided length would overflow the maximum byte size of {}.
                This is probably a programming error, or a change in the
                packet definition.",
                MAX_PACKET_SIZE
            );
        }

        Ok(n as u8)
    }

    /// Decode the length of the next packet in the buffer src, ensuring that it does
    /// not exceed limits.
    #[tracing::instrument]
    fn decode_length(&self, src: &BytesMut) -> io::Result<Option<usize>> {
        if src.len() < MIN_PACKET_SIZE {
            // Not enough data for even the header
            return Ok(None);
        }

        // get the size of this packet
        let n = match src.first() {
            Some(n) => (*n as usize) * 4,
            None => return Ok(None),
        };

        // does this exceed the max possible packet?
        if n > MAX_PACKET_SIZE {
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
        let mut codec = Codec::new();
        codec.feed(
            // Packet::Tiny, subtype TinyType::Ping, compressed, reqi=2
            &[1, 3, 2, 3],
        );
        let data = codec.decode();
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

        let codec = Codec::new();
        let buf = codec.encode(&Packet::Tiny(Tiny {
            subt: TinyType::Ping,
            reqi: RequestId(2),
        }));
        assert_ok!(&buf);

        assert_eq!(&mock[..], &buf.unwrap()[..])
    }
}
