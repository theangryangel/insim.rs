use bytes::BytesMut;
use crate::result::Result;

pub mod framed;
pub mod tcp;
pub mod udp;
pub mod websocket;

pub use framed::Framed;

#[async_trait::async_trait]
pub trait Network: Sized {
    async fn try_read_bytes(&mut self, buf: &mut BytesMut) -> Result<usize>;
    async fn try_write_bytes(&mut self, src: &[u8]) -> Result<usize>;
}
