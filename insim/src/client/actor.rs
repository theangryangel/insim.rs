use super::{Command, Config, Event, State};
use crate::{
    error::Error,
    protocol::{insim, relay, transport::Transport, Packet, VERSION},
};
use flume;
use futures_util::{SinkExt, StreamExt};
use std::sync::{Arc, Mutex};
use tokio::net::TcpStream;
use tokio::time;
use tracing;

const BACKOFF_MIN_INTERVAL_SECS: u64 = 2;
const BACKOFF_MAX_INTERVAL_SECS: u64 = 60;

pub struct ClientActor {
    pub(crate) config: Config,
    pub(crate) attempt: u64,

    pub(crate) receiver: flume::Receiver<Command>,
    pub(crate) sender: flume::Sender<Event>,

    pub(crate) state: Arc<Mutex<State>>,
}

impl ClientActor {
    async fn connect(&mut self) -> std::io::Result<TcpStream> {
        {
            let mut state = self.state.lock().unwrap();
            *state = State::Connecting;
        }

        self.sender
            .send(Event::State(State::Connecting))
            .expect("failed to send Event::Connecting");
        tracing::debug!("connecting...");

        self.attempt += 1;
        TcpStream::connect(self.config.host.to_owned()).await
    }

    async fn handshake(&mut self, stream: TcpStream) -> Result<Transport<TcpStream>, Error> {
        tracing::debug!("handshaking...");

        {
            let mut state = self.state.lock().unwrap();
            *state = State::Handshaking;
        }

        self.sender
            .send(Event::State(State::Handshaking))
            .expect("failed to send Event::Handshaking");

        let mut inner = Transport::new(stream, self.config.codec_mode);

        if self.config.verify_version {
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
                        if insimver != VERSION {
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
        }

        // TODO: can we generalise this?
        if let Some(hname) = self.config.select_relay_host.as_deref() {
            if let Err(e) = inner
                .send(
                    relay::HostSelect {
                        hname: hname.into(),
                        ..Default::default()
                    }
                    .into(),
                )
                .await
            {
                return Err(e.into());
            }

            match inner.next().await {
                Some(Ok(Packet::RelayError(relay::Error { err: e, .. }))) => {
                    return Err(Error::RelayError(e));
                }
                Some(Ok(_)) => {
                    // TODO: we're dropping a frame here
                }
                Some(Err(e)) => {
                    tracing::debug!("got error {:?} during relay host selection", e);
                    return Err(e);
                }
                None => {
                    tracing::debug!("relay host selection timed out");
                    return Err(Error::Timeout);
                }
            }
        }

        Ok(inner)
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
                {
                    let mut state = self.state.lock().unwrap();
                    *state = State::Shutdown;
                }

                self.sender
                    .send(Event::State(State::Shutdown))
                    .expect("failed to send Event::State(State::Shutdown) after backoff");
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

            {
                let mut state = self.state.lock().unwrap();
                *state = State::Connected;
            }

            self.sender
                .send(Event::State(State::Connected))
                .expect("failed to send Event::State(State::Connected))");

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
                        },

                        _ => {}
                    },

                    m = self.receiver.recv_async() => match m {

                        Ok(Command::Frame(frame)) => {
                            transport.send(frame).await.expect("failed to transmit frame");
                        },

                        Ok(Command::Shutdown) => {
                            tracing::debug!("received shutdown request");
                            {
                                let mut state = self.state.lock().unwrap();
                                *state = State::Shutdown;
                            }

                            return;
                        },

                        Err(e) => {
                            tracing::debug!("flume error: {:?}", e);
                            return
                        },

                    }

                }
            }

            {
                let mut state = self.state.lock().unwrap();
                *state = State::Disconnected;
            }
        }
    }
}
