//! TCP/UDP/Websocket bindings, and adapters to receive and send

use crate::result::Result;
use bytes::BytesMut;

pub(crate) mod bufwriter;
pub(crate) mod codec;
pub(crate) mod framed;
pub(crate) mod mode;
pub(crate) mod tcp;
pub(crate) mod udp;

#[cfg(feature = "websocket")]
pub(crate) mod websocket;

pub use codec::Codec;
pub use framed::{Framed, FramedInner};
pub use mode::Mode;

/// If no data is received within this period of seconds, consider the Insim connection to be lost.
pub const DEFAULT_TIMEOUT_SECS: u64 = 90;

/// Think of this as our own AsyncRead and AsyncWrite.
#[async_trait::async_trait]
pub trait TryReadWriteBytes: Send + Sync + Sized {
    async fn try_read_bytes(&mut self, buf: &mut BytesMut) -> Result<usize>;
    async fn try_write_bytes(&mut self, src: &[u8]) -> Result<usize>;
}
