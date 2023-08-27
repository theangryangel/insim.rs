use crate::packets::Packet;
use crate::result::Result;

#[async_trait::async_trait]
pub trait ReadPacket {
    /// Read a packet
    async fn read(&mut self) -> Result<Option<Packet>>;
}

#[async_trait::async_trait]
pub trait WritePacket {
    /// Write a packet
    async fn write(&mut self, packet: Packet) -> Result<()>;
}

pub trait ReadWritePacket: ReadPacket + WritePacket {
    fn boxed<'a>(self) -> Box<dyn ReadWritePacket + 'a>
    where
        Self: Sized + 'a,
    {
        Box::new(self)
    }
}
