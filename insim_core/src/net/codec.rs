use std::fmt;

use bytes::{Bytes, BytesMut};

/// Network framing and I/O errors.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// An operating system error.
    #[error(transparent)]
    Io(#[from] std::io::Error),
    /// Packet encoding failed.
    #[error(transparent)]
    Encode(#[from] crate::EncodeError),
    /// The stream has closed.
    #[error("disconnected")]
    Disconnected,
    /// Invalid framing or a datagram not recognized by any decoder.
    #[error("invalid frame: {0}")]
    Frame(&'static str),
    /// A complete frame could not be decoded.
    #[error("decode error at offset {offset}: {error}")]
    Decode {
        /// Original frame.
        input: Bytes,
        /// Byte offset of the failure.
        offset: usize,
        /// Packet decoder error.
        #[source]
        error: crate::DecodeError,
    },
    /// A frame decoded successfully but left bytes unread, which usually means a
    /// packet definition does not match the peer. The frame has been consumed, so the
    /// stream remains usable.
    #[error("partial decode: {} of {} bytes unread", remaining.len(), input.len())]
    IncompleteDecode {
        /// Original frame.
        input: Bytes,
        /// Bytes left after decoding.
        remaining: Bytes,
    },
}

/// Result of network or framing operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Translates bytes into packets, with no protocol lifecycle or I/O state.
pub trait Decoder: fmt::Debug + Send + 'static {
    /// Packet produced by this decoder.
    type Item: fmt::Debug + Send + 'static;
    /// Whether this decoder claims a complete datagram. Claims are tried in order.
    /// A claimed malformed datagram is an error, never a fallback to another decoder.
    fn accepts(&self, datagram: &[u8]) -> bool;
    /// Decode one frame. Incomplete input must return `None` without consuming bytes.
    fn decode(&mut self, input: &mut BytesMut) -> Result<Option<Self::Item>>;
}

/// Translates packets into framed bytes.
pub trait Encoder: Decoder {
    /// Encode atomically: failures must not append partial frames to `output`.
    fn encode(&self, packet: &Self::Item, output: &mut BytesMut) -> Result<()>;
    /// Length of the first queued frame, used to preserve UDP datagram boundaries.
    fn frame_len(&self, output: &[u8]) -> Result<usize>;
}

struct Mapped<D, T> {
    decoder: D,
    marker: std::marker::PhantomData<fn() -> T>,
}
impl<D: fmt::Debug, T> fmt::Debug for Mapped<D, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.decoder.fmt(f)
    }
}
impl<D: Decoder, T: fmt::Debug + Send + 'static> Decoder for Mapped<D, T>
where
    D::Item: Into<T>,
{
    type Item = T;
    fn accepts(&self, input: &[u8]) -> bool {
        self.decoder.accepts(input)
    }
    fn decode(&mut self, input: &mut BytesMut) -> Result<Option<T>> {
        self.decoder.decode(input).map(|p| p.map(Into::into))
    }
}

/// Ordered decoder selection for protocols sharing one UDP socket.
///
/// Formats without unique discriminators can overlap. The first decoder claiming a
/// datagram wins; use explicit ordering or separate ports for ambiguous formats.
/// For InSim, see `insim::net::Demux`.
#[derive(Debug)]
pub struct Demux<T: fmt::Debug> {
    decoders: Vec<Box<dyn Decoder<Item = T>>>,
}
impl<T: fmt::Debug + Send + 'static> Default for Demux<T> {
    fn default() -> Self {
        Self::new()
    }
}
impl<T: fmt::Debug + Send + 'static> Demux<T> {
    /// An empty decoder stack.
    pub fn new() -> Self {
        Self {
            decoders: Vec::new(),
        }
    }
    /// Append a decoder, converting its packets into the stack's output type.
    pub fn with<D: Decoder>(mut self, decoder: D) -> Self
    where
        D::Item: Into<T>,
    {
        self.decoders.push(Box::new(Mapped {
            decoder,
            marker: std::marker::PhantomData,
        }));
        self
    }
}
impl<T: fmt::Debug + Send + 'static> Decoder for Demux<T> {
    type Item = T;
    fn accepts(&self, input: &[u8]) -> bool {
        self.decoders.iter().any(|d| d.accepts(input))
    }
    fn decode(&mut self, input: &mut BytesMut) -> Result<Option<T>> {
        match self.decoders.iter_mut().find(|d| d.accepts(input)) {
            Some(d) => d.decode(input),
            None => Err(Error::Frame("no decoder recognizes datagram")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[derive(Debug)]
    struct Claimed(bool);
    impl Decoder for Claimed {
        type Item = u8;
        fn accepts(&self, _: &[u8]) -> bool {
            true
        }
        fn decode(&mut self, input: &mut BytesMut) -> Result<Option<u8>> {
            if !self.0 {
                return Err(Error::Frame("claimed but malformed"));
            }
            let value = input[0];
            input.clear();
            Ok(Some(value))
        }
    }
    #[test]
    fn recognized_malformed_datagram_does_not_fall_through() {
        let mut demux = Demux::<u8>::new().with(Claimed(false)).with(Claimed(true));
        let mut input = BytesMut::from(&[7][..]);
        assert!(matches!(
            demux.decode(&mut input),
            Err(Error::Frame("claimed but malformed"))
        ));
        assert_eq!(input.as_ref(), &[7]);
    }
}
