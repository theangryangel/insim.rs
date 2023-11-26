use bytes::BytesMut;
use tokio::net::UdpSocket;

use crate::error::Error;
use crate::result::Result;

use super::Network;

#[async_trait::async_trait]
impl Network for UdpSocket {
    async fn try_read_bytes(&mut self, buf: &mut BytesMut) -> Result<usize> {
        loop {
            let ready = self.ready(tokio::io::Interest::READABLE).await?;

            if ready.is_readable() {
                // Tokio docs indicates that the buffer must be large enough for any packet.
                // I've picked 1492 because its the effectively a common MTU size across the internet
                // still, and should give some future proofing if any packets insim increase
                // in size
                if buf.capacity() < 1492 {
                    buf.reserve(1492 - buf.capacity());
                }

                match self.try_recv_buf(buf) {
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
    }

    async fn try_write_bytes(&mut self, src: &[u8]) -> Result<usize> {
        if src.is_empty() {
            return Ok(0);
        }

        loop {
            self.writable().await?;

            match self.try_send(src) {
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