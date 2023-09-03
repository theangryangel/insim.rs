//! Connection maintains a connection and provides a Stream and Sink of
//! [Packets](crate::packets::Packet).

mod event;
mod options;
mod r#type;

pub use event::Event;

pub use options::{ConnectionOptions, ReconnectOptions};

use crate::{
    error::Error,
    packets::Packet,
    result::Result,
    tools::{handshake, maybe_keepalive},
    traits::ReadWritePacket,
};

use std::time::Duration;

pub struct Connection {
    options: ConnectionOptions,
    network: Option<Box<dyn ReadWritePacket>>,
    shutdown: bool,
}

impl Connection {
    pub fn new(options: ConnectionOptions) -> Self {
        Self {
            network: None,
            options,
            shutdown: false,
        }
    }

    pub async fn send<P: Into<Packet>>(&mut self, packet: P) -> Result<()> {
        if self.network.is_none() {
            return Err(Error::Disconnected);
        }

        let stream = self.network.as_mut().unwrap();
        stream.write(packet.into()).await
    }

    pub async fn shutdown(&mut self) {
        self.shutdown = true;
    }

    pub async fn poll(&mut self) -> Result<Event> {
        // TODO handle ReconnectOptions
        if self.network.is_none() {
            let isi = self.options.as_isi();
            let stream = self
                .options
                .transport
                .connect(isi, Duration::from_secs(90))
                .await?;

            self.network = Some(stream);
            return Ok(Event::Connected);
        }

        self.poll_inner().await
    }

    async fn poll_inner(&mut self) -> Result<Event> {
        let stream = self.network.as_mut().unwrap();

        match stream.read().await? {
            Some(packet) => {
                maybe_keepalive(stream, &packet).await?;
                Ok(Event::Data(packet))
            }
            None => {
                self.network = None;
                // TODO: is this really true?
                Ok(Event::Disconnected)
            }
        }
    }
}
