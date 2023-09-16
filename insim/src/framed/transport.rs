use bytes::{BytesMut, Bytes};
use crate::result::Result;

#[async_trait::async_trait]
pub trait Transport: Sized {
    async fn try_read_bytes(&mut self, buf: &mut BytesMut) -> Result<usize>;
    async fn try_write_bytes(&mut self, src: &[u8]) -> Result<usize>;
}
