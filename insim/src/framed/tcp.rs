use bytes::{BytesMut, Bytes};
use tokio::{net::TcpStream, io::AsyncWriteExt};
use crate::{error::Error, result::Result};

use super::transport::{Transport, IntoFramed};

pub struct Tcp {
    pub inner: TcpStream
}

#[async_trait::async_trait]
impl Transport for Tcp {
    async fn read(&mut self, buf: &mut BytesMut) -> Result<usize> {
        loop {
            self.inner.readable().await?;

            match self.inner.try_read_buf(buf) {
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

    async fn write(&mut self, src: &mut BytesMut) -> Result<()> {
        self.inner.write_buf(src).await?;
        Ok(())
    }
}

impl IntoFramed for Tcp {}

impl From<TcpStream> for Tcp {
    fn from(inner: TcpStream) -> Self {
        Self { inner }
    }
}
