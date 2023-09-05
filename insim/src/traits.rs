use crate::packets::Packet;
use crate::result::Result;

#[async_trait::async_trait]
pub trait ReadPacket {
    /// Read a packet
    async fn read(&mut self) -> Result<Packet>;
}

#[async_trait::async_trait]
pub trait WritePacket {
    /// Write a packet
    async fn write(&mut self, packet: Packet) -> Result<()>;
}

pub trait ReadWritePacket: ReadPacket + WritePacket + Send {
    fn boxed<'a>(self) -> Box<dyn ReadWritePacket + 'a>
    where
        Self: Sized + 'a,
    {
        Box::new(self)
    }
}

#[async_trait::async_trait]
impl<I: ReadPacket + Send + ?Sized> ReadPacket for Box<I> {
    async fn read(&mut self) -> Result<Packet> {
        (**self).read().await
    }
}

#[async_trait::async_trait]
impl<I: WritePacket + Send + ?Sized> WritePacket for Box<I> {
    async fn write(&mut self, packet: Packet) -> Result<()> {
        (**self).write(packet).await
    }
}

impl<I: ReadWritePacket + Send + ?Sized> ReadWritePacket for Box<I> {}
