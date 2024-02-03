use bytes::BytesMut;

use crate::result::Result;

pub(crate) mod bufwriter;
pub(crate) mod framed;
pub(crate) mod tcp;
pub(crate) mod udp;

#[cfg(feature = "websocket")]
pub(crate) mod websocket;

pub use framed::{Framed, FramedInner};

/// Think of this as our own AsyncRead and AsyncWrite.
#[async_trait::async_trait]
pub trait AsyncTryReadWriteBytes: Send + Sync + Sized {
    /// Try to read bytes.
    async fn try_read_bytes(&mut self, buf: &mut BytesMut) -> Result<usize>;

    /// Try to write bytes.
    async fn try_write_bytes(&mut self, src: &[u8]) -> Result<usize>;
}
