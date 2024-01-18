use crate::result::Result;
use bytes::BytesMut;

pub mod bufwriter;
pub mod codec;
pub mod framed;
pub mod mode;
pub mod tcp;
pub mod udp;
#[cfg(feature = "websocket")]
pub mod websocket;

pub use codec::Codec;
pub use framed::{Framed, FramedInner};
pub use mode::Mode;

pub const DEFAULT_TIMEOUT_SECS: u64 = 90;

#[async_trait::async_trait]
pub trait TryReadWriteBytes: Send + Sync + Sized {
    async fn try_read_bytes(&mut self, buf: &mut BytesMut) -> Result<usize>;
    async fn try_write_bytes(&mut self, src: &[u8]) -> Result<usize>;
}
