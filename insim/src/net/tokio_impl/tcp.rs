use bytes::BytesMut;
use tokio::net::TcpStream;

use super::AsyncTryReadWriteBytes;
use crate::{error::Error, result::Result};

#[async_trait::async_trait]
impl AsyncTryReadWriteBytes for TcpStream {
    async fn try_read_bytes(&mut self, buf: &mut BytesMut) -> Result<usize> {
        loop {
            self.readable().await?;

            match self.try_read_buf(buf) {
                Ok(0) => {
                    return Err(Error::Disconnected);
                },
                Ok(size) => {
                    return Ok(size);
                },
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    continue;
                },
                Err(e) => {
                    return Err(e.into());
                },
            }
        }
    }

    async fn try_write_bytes(&mut self, src: &[u8]) -> Result<usize> {
        loop {
            self.writable().await?;

            match self.try_write(src) {
                Ok(n) => {
                    return Ok(n);
                },
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => continue,
                Err(e) => {
                    return Err(e.into());
                },
            }
        }
    }
}
