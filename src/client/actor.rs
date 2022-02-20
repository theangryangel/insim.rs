use super::{Config, Event};
use crate::{
    error::Error,
    protocol::{insim, transport::Transport, Packet},
};
use flume;
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio::time;
use tracing;

const BACKOFF_MIN_INTERVAL_SECS: u64 = 2;
const BACKOFF_MAX_INTERVAL_SECS: u64 = 60;

pub struct ClientActor {
    pub(crate) config: Config,
    pub(crate) attempt: u64,

    pub(crate) receiver: flume::Receiver<Event>,
    pub(crate) sender: flume::Sender<Event>,
}

impl ClientActor {
    async fn connect(&mut self) -> std::io::Result<TcpStream> {
        self.sender
            .send(Event::Connecting)
            .expect("failed to send Event::Connecting");
        tracing::debug!("connecting...");

        self.attempt += 1;
        TcpStream::connect(self.config.host.to_owned()).await
    }

    async fn handshake(&mut self, stream: TcpStream) -> Result<Transport<TcpStream>, Error> {
        tracing::debug!("handshaking...");
        self.sender
            .send(Event::Handshaking)
            .expect("failed to send Event::Handshaking");
        let mut inner = Transport::new(stream, self.config.codec_mode);

        if !self.config.verify_version {
            return Ok(inner);
        }

        if let Err(e) = inner
            .send(
                insim::Tiny {
                    reqi: 1,
                    subtype: insim::TinyType::Version,
                }
                .into(),
            )
            .await
        {
            return Err(e.into());
        }

        while let Some(m) = inner.next().await {
            match m {
                Ok(Packet::Version(insim::Version { insimver, .. })) => {
                    if insimver != insim::VERSION {
                        return Err(Error::IncompatibleVersion);
                    }

                    return Ok(inner);
                }
                Err(e) => {
                    tracing::debug!("got error {:?} during handshake", e);
                    return Err(e);
                }
                m => {
                    tracing::debug!("ignoring {:?} until handshake is complete", m);
                }
            }
        }

        Err(Error::Timeout)
    }

    async fn backoff(&mut self) -> Result<(), Error> {
        if self.attempt == 0 {
            return Ok(());
        }

        tracing::debug!("backoff, attempt {}", self.attempt);

        if !self.config.reconnect
            || (self.config.max_reconnect_attempts > 0
                && self.attempt > self.config.max_reconnect_attempts)
        {
            tracing::debug!("skipping reconnect, max attempts reached!");
            return Err(Error::MaxConnectionAttempts);
        }

        let secs = ::std::cmp::min(
            self.attempt * BACKOFF_MIN_INTERVAL_SECS,
            BACKOFF_MAX_INTERVAL_SECS,
        );

        tracing::debug!("backing off for {}s", secs);

        tokio::time::sleep(time::Duration::from_secs(secs)).await;

        Ok(())
    }

    pub async fn run(&mut self) {
        loop {
            let backoff = self.backoff().await;
            if backoff.is_err() {
                self.sender
                    .send(Event::Shutdown)
                    .expect("failed to send Event::Shutdown after backoff");
                return;
            }

            let tcp = match self.connect().await {
                Ok(tcp) => tcp,
                Err(e) => match self.sender.send(Event::Error(e.into())) {
                    Ok(_) => {
                        continue;
                    }
                    Err(e) => {
                        tracing::error!("failed to send error {}", e);
                        return;
                    }
                },
            };

            let mut transport = match self.handshake(tcp).await {
                Ok(transport) => transport,
                Err(e) => {
                    self.sender
                        .send(Event::Error(e))
                        .expect("failed to send error after handshake");
                    continue;
                }
            };

            self.sender
                .send(Event::Connected)
                .expect("failed to send Event::Connected");

            // reset the attempt counter so that if we reconnect later the backoff is reset
            self.attempt = 0;

            loop {
                tokio::select! {

                    m = transport.next() => match m {
                        Some(Ok(m)) => {
                            self.sender.send(Event::Frame(m)).expect("failed to send Event::Frame");
                        },
                        None => {
                            tracing::debug!("disconnected");
                            break;
                        }
                        _ => {}
                    },

                    m = self.receiver.recv_async() => match m {

                        Ok(Event::Frame(frame)) => {
                            transport.send(frame).await.expect("failed to transmit frame");
                        },

                        Ok(Event::Shutdown) => {
                            tracing::debug!("received shutdown request");
                            return;
                        },

                        _ => {
                            unimplemented!()
                        }

                    }

                }
            }
        }
    }
}
