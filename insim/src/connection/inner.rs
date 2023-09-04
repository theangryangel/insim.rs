use crate::{
    packets::Packet,
    result::Result,
    traits::{ReadPacket, ReadWritePacket, WritePacket},
};

// The "Inner" connection for Connection, so that we can avoid Box'ing
// Since the ConnectionOptions is all very hard coded, for "high level" API usage,
// I think this fine.
// i.e. if we add a Websocket option down the line, then ConnectionOptions needs to understand it
// therefore we cannot just box stuff magically anyway.
pub(crate) enum ConnectionInner {
    Tcp(crate::tcp::Tcp),
    Udp(crate::udp::Udp),
}

#[async_trait::async_trait]
impl ReadPacket for ConnectionInner {
    async fn read(&mut self) -> Result<Option<Packet>> {
        match self {
            Self::Tcp(i) => i.read().await,
            Self::Udp(i) => i.read().await,
        }
    }
}

#[async_trait::async_trait]
impl WritePacket for ConnectionInner {
    async fn write(&mut self, packet: Packet) -> Result<()> {
        match self {
            Self::Tcp(i) => i.write(packet).await,
            Self::Udp(i) => i.write(packet).await,
        }
    }
}

impl ReadWritePacket for ConnectionInner {}
