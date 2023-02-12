use super::config::Config;
use crate::error::Error;
use crate::protocol::{transport::Transport, Packet};

use futures::{FutureExt, Sink, Stream, TryStreamExt};
use futures_util::sink::SinkExt;
use insim_core::identifiers::RequestId;
use pin_project::pin_project;
use std::fmt::Display;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::net::TcpStream;

#[derive(Debug, Clone)]
pub enum Event {
    Handshaking,
    Connected,
    Disconnected,
    Data(Packet),
    Error(Error),
    Shutdown,
}

impl Event {
    pub fn name(&self) -> &str {
        match self {
            Event::Handshaking => "handshaking",
            Event::Connected => "connected",
            Event::Disconnected => "disconnected",
            Event::Data(_) => "data",
            Event::Error(_) => "error",
            Event::Shutdown => "shutdown",
        }
    }
}

impl From<Packet> for Event {
    fn from(value: Packet) -> Self {
        Self::Data(value)
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum ConnectedState {
    Handshake,
    Handshaking,
    Connected,
}

impl Display for ConnectedState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConnectedState::Handshake => write!(f, "Handshake"),
            ConnectedState::Handshaking => write!(f, "Handshaking"),
            ConnectedState::Connected => write!(f, "Connected"),
        }
    }
}

type TransportConnecting = Pin<
    Box<
        dyn futures::Future<
                Output = Result<Result<TcpStream, std::io::Error>, tokio::time::error::Elapsed>,
            > + Send,
    >,
>;

#[pin_project(project = ClientStateProject)]
pub enum ClientState {
    Disconnected,
    Connecting(#[pin] TransportConnecting),
    Connected {
        #[pin]
        transport: Transport<TcpStream>,
        state: ConnectedState,
    },
    Shutdown,
}

impl Display for ClientState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClientState::Disconnected => write!(f, "Disconnected"),
            ClientState::Connecting(_) => write!(f, "Connecting"),
            ClientState::Connected { state, .. } => write!(f, "Connected ({})", state),
            ClientState::Shutdown => write!(f, "Shutdown"),
        }
    }
}

#[pin_project]
pub struct Client {
    config: Config,
    #[pin]
    inner: ClientState,
    attempts: usize,
}

impl Client {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            inner: ClientState::Disconnected,
            attempts: 0,
        }
    }

    pub fn shutdown(&mut self) {
        self.inner = ClientState::Shutdown;
    }
}

impl Stream for Client {
    type Item = Event;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        tracing::trace!(
            "poll_next: state={}, attempts={}",
            self.inner,
            self.attempts
        );
        let this = self.as_mut().project();

        // FIXME we can/should delegate some of this to ClientState and call a poll_*
        // method there to simplify the code here

        match this.inner.project() {
            ClientStateProject::Shutdown => Poll::Ready(None),
            ClientStateProject::Disconnected => {
                let attempts = *this.attempts;
                if attempts > 0
                    && (!this.config.reconnect || attempts >= this.config.max_reconnect_attempts)
                {
                    self.project().inner.set(ClientState::Shutdown);
                    return Poll::Ready(Some(Event::Shutdown));
                }

                *this.attempts += 1;
                let future = Box::pin(tokio::time::timeout(
                    std::time::Duration::from_millis(this.config.connect_timeout_ms),
                    TcpStream::connect(this.config.host.clone()),
                ));
                self.project().inner.set(ClientState::Connecting(future));
                cx.waker().wake_by_ref();
                Poll::Pending
            }
            ClientStateProject::Connecting(mut fut) => match fut.poll_unpin(cx) {
                Poll::Ready(Ok(Ok(stream))) => {
                    let transport = Transport::new(stream, this.config.codec_mode);
                    *this.attempts = 1;
                    self.project().inner.set(ClientState::Connected {
                        transport,
                        state: ConnectedState::Handshake,
                    });
                    cx.waker().wake_by_ref();
                    Poll::Pending
                }
                Poll::Ready(Ok(Err(e))) => {
                    // connection failed, did not timeout
                    self.project().inner.set(ClientState::Disconnected);
                    cx.waker().wake_by_ref();
                    Poll::Ready(Some(Event::Error(e.into())))
                }
                Poll::Ready(Err(e)) => {
                    // connection timed out
                    self.project().inner.set(ClientState::Disconnected);
                    cx.waker().wake_by_ref();
                    Poll::Ready(Some(Event::Error(Error::IO {
                        kind: std::io::ErrorKind::TimedOut,
                        message: format!("Initial connection timeout: {}", e),
                    })))
                }
                Poll::Pending => Poll::Pending,
            },
            ClientStateProject::Connected {
                ref mut transport,
                state,
            } => match state {
                ConnectedState::Handshake => {
                    if transport.poll_ready_unpin(cx).is_pending() {
                        return Poll::Pending;
                    }

                    let isi = crate::protocol::insim::Init {
                        name: this.config.name.to_owned(),
                        password: this.config.password.to_owned(),
                        prefix: b'!',
                        version: crate::protocol::VERSION,
                        interval: this.config.interval_ms,
                        flags: this.config.flags,
                        reqi: RequestId(1),
                    };

                    if let Err(e) = transport.start_send_unpin(isi.into()) {
                        self.project().inner.set(ClientState::Disconnected);
                        return Poll::Ready(Some(Event::Error(e)));
                    }

                    if let Err(e) = futures::ready!(transport.poll_flush_unpin(cx)) {
                        self.project().inner.set(ClientState::Disconnected);
                        return Poll::Ready(Some(Event::Error(e)));
                    }

                    if let Some(host) = &this.config.select_relay_host {
                        let select = crate::protocol::relay::HostSelect {
                            hname: host.to_owned(),
                            ..Default::default()
                        };

                        if let Err(e) = transport.start_send_unpin(select.into()) {
                            self.project().inner.set(ClientState::Disconnected);
                            return Poll::Ready(Some(Event::Error(e)));
                        }

                        if let Err(e) = futures::ready!(transport.poll_flush_unpin(cx)) {
                            self.project().inner.set(ClientState::Disconnected);
                            return Poll::Ready(Some(Event::Error(e)));
                        }
                    }

                    if !this.config.verify_version {
                        *state = ConnectedState::Connected;
                        Poll::Ready(Some(Event::Connected))
                    } else {
                        *state = ConnectedState::Handshaking;
                        Poll::Ready(Some(Event::Handshaking))
                    }
                }
                ConnectedState::Handshaking => match transport.try_poll_next_unpin(cx) {
                    Poll::Ready(Some(packet)) => {
                        if let Ok(Packet::RelayError(crate::protocol::relay::RelayError {
                            err,
                            ..
                        })) = packet
                        {
                            self.project().inner.set(ClientState::Disconnected);
                            return Poll::Ready(Some(Event::Error(Error::Relay(err))));
                        }

                        match (packet, this.config.verify_version) {
                            (Ok(packet), true) => match packet {
                                Packet::Version(crate::protocol::insim::Version {
                                    insimver,
                                    ..
                                }) => {
                                    if insimver != crate::protocol::VERSION {
                                        return Poll::Ready(Some(Event::Error(
                                            Error::IncompatibleVersion(insimver),
                                        )));
                                    }

                                    // TODO: automatically poll server for connected players, etc.
                                    *state = ConnectedState::Connected;
                                    Poll::Ready(Some(Event::Connected))
                                }
                                _ => Poll::Pending,
                            },
                            (Ok(_), false) => {
                                *state = ConnectedState::Connected;
                                Poll::Ready(Some(Event::Connected))
                            }
                            (Err(e), _) => {
                                self.project().inner.set(ClientState::Disconnected);
                                Poll::Ready(Some(Event::Error(e)))
                            }
                        }
                    }
                    Poll::Ready(None) => {
                        self.project().inner.set(ClientState::Disconnected);
                        Poll::Ready(Some(Event::Disconnected))
                    }
                    Poll::Pending => Poll::Pending,
                },
                ConnectedState::Connected => match transport.try_poll_next_unpin(cx) {
                    Poll::Ready(Some(packet)) => match packet {
                        Ok(packet) => Poll::Ready(Some(Event::Data(packet))),
                        Err(e) => {
                            self.project().inner.set(ClientState::Disconnected);
                            Poll::Ready(Some(Event::Error(e)))
                        }
                    },
                    Poll::Ready(None) => {
                        self.project().inner.set(ClientState::Disconnected);
                        Poll::Ready(Some(Event::Disconnected))
                    }
                    Poll::Pending => Poll::Pending,
                },
            },
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        fn twice_plus_one(value: Option<usize>) -> Option<usize> {
            value?.checked_mul(2)?.checked_add(1)
        }

        match &self.inner {
            ClientState::Connected { transport, .. } => {
                let (lower, upper) = transport.size_hint();
                (lower, twice_plus_one(upper))
            }
            _ => (0, Some(1)), // there may be at least 1 event pending if we're not connected
        }
    }
}

impl Sink<Event> for Client {
    type Error = Error;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        match self.project().inner.project() {
            ClientStateProject::Disconnected => Poll::Pending,
            ClientStateProject::Connecting(_) => Poll::Pending,
            ClientStateProject::Connected {
                ref mut transport, ..
            } => transport.poll_ready_unpin(cx),
            ClientStateProject::Shutdown => Poll::Ready(Ok(())),
        }
    }

    fn start_send(mut self: Pin<&mut Self>, item: Event) -> Result<(), Self::Error> {
        let mut this = self.as_mut().project();

        if matches!(item, Event::Shutdown) {
            tracing::debug!("Fuck?");
            this.inner.set(ClientState::Shutdown);
        };

        match this.inner.project() {
            ClientStateProject::Disconnected => Err(Error::Disconnected),
            ClientStateProject::Connecting(_) => Err(Error::Disconnected),
            ClientStateProject::Connected {
                ref mut transport, ..
            } => match item {
                Event::Data(packet) => transport.start_send_unpin(packet),
                Event::Connected => todo!(),
                Event::Disconnected => todo!(),
                Event::Error(_) => todo!(),
                Event::Shutdown => todo!(),
                Event::Handshaking => todo!(),
            },
            ClientStateProject::Shutdown => {
                self.project().inner.set(ClientState::Shutdown);
                Ok(())
            }
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        match self.project().inner.project() {
            ClientStateProject::Disconnected => Poll::Pending,
            ClientStateProject::Connecting(_) => Poll::Pending,
            ClientStateProject::Connected {
                ref mut transport, ..
            } => transport.poll_flush_unpin(cx),
            ClientStateProject::Shutdown => Poll::Ready(Ok(())),
        }
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        match self.project().inner.project() {
            ClientStateProject::Disconnected => Poll::Ready(Ok(())),
            ClientStateProject::Connecting(_) => Poll::Pending,
            ClientStateProject::Connected {
                ref mut transport, ..
            } => transport.poll_close_unpin(cx),
            ClientStateProject::Shutdown => Poll::Ready(Ok(())),
        }
    }
}
