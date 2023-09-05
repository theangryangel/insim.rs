mod event;
mod inner;
mod network_options;
mod options;

pub use event::Event;
pub use options::ConnectionOptions;

use crate::{
    connection::inner::ConnectionInner,
    error::Error,
    packets::Packet,
    result::Result,
    tools::net::{handshake, maybe_keepalive},
    traits::{ReadPacket, WritePacket},
};

pub struct Connection {
    options: ConnectionOptions,
    inner: Option<ConnectionInner>,
    shutdown: bool,

    shutdown_notify: tokio::sync::Notify,
}

impl Connection {
    pub fn new(options: ConnectionOptions) -> Self {
        Self {
            inner: None,
            options,
            shutdown: false,
            shutdown_notify: tokio::sync::Notify::new(),
        }
    }

    pub async fn send<P: Into<Packet>>(&mut self, packet: P) -> Result<()> {
        if self.shutdown {
            return Err(Error::Shutdown);
        }

        match self.inner.as_mut() {
            None => Err(Error::Disconnected),
            Some(ref mut inner) => inner.write(packet.into()).await,
        }
    }

    pub fn shutdown(&mut self) {
        self.shutdown = true;
        self.shutdown_notify.notify_one();
    }

    /// Handles establishing the connection, managing the keepalive (ping) and returns the next
    /// [Event].
    /// On error, calling again will result in a reconnection attempt.
    /// Failure to call poll will result in a timeout.
    pub async fn poll(&mut self) -> Result<Event> {
        if self.shutdown {
            return Err(Error::Shutdown);
        }

        if self.inner.is_none() {
            let stream = self.options.connect().await?;

            self.inner = Some(stream);
            return Ok(Event::Connected);
        }

        match self.poll_inner().await {
            Ok(inner) => Ok(inner),
            Err(inner) => {
                self.inner = None;
                Err(inner)
            }
        }
    }

    async fn poll_inner(&mut self) -> Result<Event> {
        let stream = self.inner.as_mut().unwrap();

        tokio::select! {
            _ = self.shutdown_notify.notified() => {
                Ok(Event::Shutdown)
            },

            packet = stream.read() => {
                let packet = packet?;
                maybe_keepalive(stream, &packet).await?;

                if packet.is_error() {
                    Err(packet.into())
                } else {
                    Ok(Event::Data(packet))
                }
            },
        }
    }
}
