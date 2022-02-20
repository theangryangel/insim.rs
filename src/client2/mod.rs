use super::client::{Config, Event};
use super::protocol::transport::Transport;
use flume;
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio::task::JoinHandle;
use tokio::time;
use tracing;

const BACKOFF_MIN_INTERVAL_SECS: u64 = 2;
const BACKOFF_MAX_INTERVAL_SECS: u64 = 60;

pub struct Client2Inner {
    config: Config,
    attempt: u64,

    receiver: flume::Receiver<Event>,
    sender: flume::Sender<Event>,
}

impl Client2Inner {
    async fn connect(&mut self) -> std::io::Result<TcpStream> {
        tracing::debug!("connecting...");

        self.attempt += 1;
        TcpStream::connect(self.config.host.to_owned()).await
    }

    async fn handshake(&mut self, stream: TcpStream) -> Result<Transport<TcpStream>, ()> {
        tracing::debug!("handshaking...");
        let inner = Transport::new(stream, self.config.codec_mode);

        Ok(inner)
    }

    // FIXME
    async fn backoff(&mut self) -> Result<(), ()> {
        if self.attempt <= 0 {
            return Ok(());
        }

        tracing::debug!("backoff, attempt {}", self.attempt);

        if self.attempt > 0
            && (!self.config.reconnect
                || (self.config.max_reconnect_attempts > 0
                    // FIXME
                    && self.attempt > self.config.max_reconnect_attempts.try_into().unwrap()))
        {
            tracing::debug!("skipping reconnect, max attempts reached!");
            return Err(());
        }

        let secs = ::std::cmp::min(
            self.attempt * BACKOFF_MIN_INTERVAL_SECS,
            BACKOFF_MAX_INTERVAL_SECS,
        );

        tracing::debug!("sleeping for {}s", secs);

        tokio::time::sleep(time::Duration::from_secs(secs)).await;

        tracing::debug!("backoff done");
        Ok(())
    }

    async fn run(&mut self) {
        loop {
            tracing::debug!("loop");

            let backoff = self.backoff().await;
            if backoff.is_err() {
                self.sender.send(Event::Shutdown);
                return;
            }

            let tcp = match self.connect().await {
                Ok(tcp) => tcp,
                Err(_e) => {
                    // FIXME - return an error
                    continue;
                }
            };

            // FIXME
            let mut transport = match self.handshake(tcp).await {
                Ok(transport) => transport,
                _ => {
                    continue;
                }
            };

            self.sender.send(Event::Connected);
            // reset the attempt counter so that if we reconnect later the backoff is reset
            self.attempt = 0;

            loop {
                tokio::select! {

                    m = transport.next() => match m {
                        Some(Ok(m)) => {
                            tracing::debug!("{:?}", m);
                            self.sender.send(Event::Packet(m));
                        },
                        None => {
                            tracing::debug!("disconnected");
                            break;
                        }
                        _ => {}
                    },

                    m = self.receiver.recv_async() => match m {

                        Ok(Event::Packet(frame)) => {
                            transport.send(frame).await;
                        },

                        Ok(Event::Shutdown) => {
                            // FIXME
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

pub struct Client2 {
    receiver: flume::Receiver<Event>,
    sender: flume::Sender<Event>,

    handle: JoinHandle<()>,
}

impl Client2 {
    pub fn from_config(config: Config) -> Self {
        let (actor_tx, actor_rx) = flume::unbounded();
        let (client_tx, client_rx) = flume::unbounded();

        let mut actor = Client2Inner {
            config,
            receiver: client_rx,
            sender: actor_tx,
            attempt: 0,
        };

        let handle = tokio::spawn(async move {
            actor.run().await;
        });

        Self {
            receiver: actor_rx,
            sender: client_tx,
            handle,
        }
    }

    pub fn send(&self, e: Event) {
        // FIXME
        self.sender.send(e);
    }

    pub async fn recv(&self) -> Option<Event> {
        // FIXME
        match self.receiver.recv_async().await {
            Ok(e) => Some(e),

            _ => None,
        }
    }

    pub async fn shutdown(self) {
        self.send(Event::Shutdown);
        // FIXME
        self.handle.await;
    }
}
