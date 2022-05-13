use super::{service::Service, Command, Config, Event, State};
use crate::{
    error::Error,
    protocol::{insim, relay, transport::Transport, Packet, VERSION},
};
use futures_util::{SinkExt, StreamExt};
use std::pin::Pin;
use std::{
    cell::Cell,
    ops::Deref,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex,
    },
};
use tokio::net::TcpStream;
use tokio::time;
use tracing;

const BACKOFF_MIN_INTERVAL_SECS: u64 = 2;
const BACKOFF_MAX_INTERVAL_SECS: u64 = 60;

pub struct Connection {
    pub(crate) config: Config,
    pub(crate) attempt: AtomicU64,

    pub(crate) actor_rx: flume::Receiver<Command>,

    pub(crate) state: Arc<Mutex<State>>,

    pub(crate) services: Vec<Box<dyn Service + Sync + Send>>,

    pub(crate) futures: futures::stream::FuturesUnordered<tokio::task::JoinHandle<()>>,
}

impl Connection {
    fn event(&mut self, event: Event) {
        println!("{:?}", event);
        for service in &self.services {
            let service = service.clone();
            let event = event.clone();
            let fut = tokio::spawn(async move {
                service.call(event).await;
            });
            self.futures.push(fut);
        }
    }

    async fn connect(&mut self) -> std::io::Result<TcpStream> {
        tracing::debug!("connecting...");

        {
            let mut state = self.state.lock().unwrap();
            *state = State::Connecting;
        }

        self.event(Event::State(State::Connecting));

        self.attempt
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |attempt| {
                Some(attempt + 1)
            });

        TcpStream::connect(self.config.host.to_owned()).await
    }

    async fn handshake(&mut self, stream: TcpStream) -> Result<Transport<TcpStream>, Error> {
        tracing::debug!("handshaking...");

        {
            let mut state = self.state.lock().unwrap();
            *state = State::Handshaking;
        }

        self.event(Event::State(State::Handshaking));

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

            return match inner.next().await {
                Some(Ok(Packet::RelayError(relay::Error { err: e, .. }))) => {
                    Err(Error::RelayError(e))
                }
                Some(Ok(_)) => {
                    // TODO: we're dropping a frame here
                    Ok(inner)
                }
                Some(Err(e)) => {
                    tracing::debug!("got error {:?} during relay host selection", e);
                    Err(e)
                }
                None => {
                    tracing::debug!("relay host selection timed out");
                    Err(Error::Timeout)
                }
            };
        } else {
            Ok(inner)
        }
    }

    async fn backoff(&mut self) -> Result<(), Error> {
        let attempt = self.attempt.load(Ordering::SeqCst);

        if attempt == 0 {
            return Ok(());
        }

        tracing::debug!("backoff, attempt {}", attempt);

        if !self.config.reconnect
            || (self.config.max_reconnect_attempts > 0
                && attempt > self.config.max_reconnect_attempts)
        {
            tracing::debug!("skipping reconnect, max attempts reached!");
            return Err(Error::MaxConnectionAttempts);
        }

        let secs = ::std::cmp::min(
            attempt * BACKOFF_MIN_INTERVAL_SECS,
            BACKOFF_MAX_INTERVAL_SECS,
        );

        tracing::debug!("backing off for {}s", secs);

        tokio::time::sleep(time::Duration::from_secs(secs)).await;

        Ok(())
    }

    pub async fn run(&mut self) -> Option<()> {
        loop {
            let backoff = self.backoff().await;
            if backoff.is_err() {
                {
                    let mut state = self.state.lock().unwrap();
                    *state = State::Shutdown;
                }

                self.event(Event::State(State::Shutdown));

                return None;
            }

            let tcp = match self.connect().await {
                Ok(tcp) => tcp,
                Err(e) => {
                    self.event(Event::Error(e.into()));
                    continue;
                }
            };

            let mut transport = match self.handshake(tcp).await {
                Ok(transport) => transport,
                Err(e) => {
                    self.event(Event::Error(e));
                    continue;
                }
            };

            {
                let mut state = self.state.lock().unwrap();
                *state = State::Connected;
            }

            self.event(Event::State(State::Connected));

            // reset the attempt counter so that if we reconnect later the backoff is reset
            self.attempt.store(0, Ordering::SeqCst);

            loop {
                tokio::select! {

                    m = transport.next() => match m {


                        Some(Ok(m)) => {
                            self.event(Event::Frame(m));
                        },

                        None => {
                            tracing::debug!("disconnected");
                            break;
                        },

                        _ => {}
                    },

                    m = self.actor_rx.recv_async() => match m {
                        Ok(Command::Frame(frame)) => {
                            transport.send(frame).await.expect("failed to transmit frame");
                        },

                        Ok(Command::Shutdown) => {
                            tracing::debug!("received shutdown request");
                            {
                                let mut state = self.state.lock().unwrap();
                                *state = State::Shutdown;
                            }

                            return None;
                        },

                        Err(e) => {
                            tracing::debug!("flume error: {:?}", e);
                            return None
                        },
                    },

                    // f = self.futures.next() => {
                    //     println!("GOT FUTURE {:?}", f);
                    // }
                }
            }

            {
                let mut state = self.state.lock().unwrap();
                *state = State::Disconnected;
            }

            self.event(Event::State(State::Disconnected));
        }
    }
}
