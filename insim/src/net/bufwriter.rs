use crate::{error::Error, result::Result};
use bytes::BytesMut;
use tokio::net::TcpStream;

use super::TryReadWriteBytes;

#[async_trait::async_trait]
impl TryReadWriteBytes for tokio::io::BufWriter<TcpStream> {
    async fn try_read_bytes(&mut self, buf: &mut BytesMut) -> Result<usize> {
        loop {
            self.get_mut().readable().await?;

            match self.get_mut().try_read_buf(buf) {
                Ok(0) => {
                    return Err(Error::Disconnected);
                }
                Ok(size) => {
                    return Ok(size);
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    continue;
                }
                Err(e) => {
                    return Err(e.into());
                }
            }
        }
    }

    async fn try_write_bytes(&mut self, src: &[u8]) -> Result<usize> {
        loop {
            self.get_mut().writable().await?;

            match self.get_mut().try_write(src) {
                Ok(n) => {
                    return Ok(n);
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => continue,
                Err(e) => {
                    return Err(e.into());
                }
            }
        }
    }
}
