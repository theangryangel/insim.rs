use super::config::Config;
use crate::error::Error;
use crate::protocol::{transport::Transport, Packet};

use futures::{FutureExt, Sink, Stream, TryStreamExt};
use futures_util::sink::SinkExt;
use pin_project::pin_project;
use std::fmt::Display;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::net::TcpStream;

#[derive(PartialEq, Debug)]
pub enum Event {
    Handshaking,
    Connected,
    Disconnected,
    Data(Packet),
    Error(Error),
    Shutdown,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum ConnectedState {
    Handshake,
    Handshaking,
    Established,
}

impl Display for ConnectedState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConnectedState::Handshake => write!(f, "Handshake"),
            ConnectedState::Handshaking => write!(f, "Handshaking"),
            ConnectedState::Established => write!(f, "Established"),
        }
    }
}

type TransportConnecting = Pin<
    Box<
        dyn futures::Future<
            Output = Result<Result<TcpStream, std::io::Error>, tokio::time::error::Elapsed>,
        >,
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

        match this.inner.project() {
            ClientStateProject::Shutdown => Poll::Ready(None),
            ClientStateProject::Disconnected => {
                if *this.attempts > 0
                    && (!this.config.reconnect
                        || *this.attempts >= this.config.max_reconnect_attempts)
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
                Poll::Ready(Err(_)) => {
                    // connection timed out
                    self.project().inner.set(ClientState::Disconnected);
                    cx.waker().wake_by_ref();
                    Poll::Ready(Some(Event::Error(Error::Timeout)))
                }
                Poll::Pending => Poll::Pending,
            },
            ClientStateProject::Connected {
                state,
                mut transport,
            } => {
                if *state == ConnectedState::Handshake {
                    let isi = crate::protocol::insim::Init {
                        name: this.config.name.to_owned(),
                        password: this.config.password.to_owned(),
                        prefix: b'!',
                        version: crate::protocol::VERSION,
                        interval: this.config.interval_ms,
                        flags: this.config.flags,
                        reqi: 1,
                    };

                    transport.start_send_unpin(isi.into()); // FIXME
                    transport.poll_flush_unpin(cx); // FIXME

                    if let Some(host) = &this.config.select_relay_host {
                        let select = crate::protocol::relay::HostSelect {
                            hname: host.to_owned().into(), // FIXME
                            ..Default::default()
                        };

                        transport.start_send_unpin(select.into()); // FIXME
                        transport.poll_flush_unpin(cx); // FIXME
                    }

                    *state = ConnectedState::Handshaking;

                    return Poll::Ready(Some(Event::Handshaking));
                }

                match transport.try_poll_next_unpin(cx) {
                    Poll::Ready(Some(packet)) => {
                        if let Ok(Packet::RelayError(crate::protocol::relay::Error {
                            err, ..
                        })) = packet
                        {
                            self.project().inner.set(ClientState::Disconnected);
                            return Poll::Ready(Some(Event::Error(Error::RelayError(err))));
                        }

                        match (packet, *state, this.config.verify_version) {
                            (Ok(_), ConnectedState::Handshake, _) => Poll::Pending, // we shouldnt really get any packets yet
                            (Ok(packet), ConnectedState::Handshaking, true) => match packet {
                                Packet::Version(crate::protocol::insim::Version {
                                    insimver,
                                    ..
                                }) => {
                                    if insimver != crate::protocol::VERSION {
                                        return Poll::Ready(Some(Event::Error(
                                            Error::IncompatibleVersion,
                                        )));
                                    }

                                    // TODO: automatically poll server for connected players, etc.
                                    *state = ConnectedState::Established;
                                    Poll::Ready(Some(Event::Connected))
                                }
                                _ => Poll::Pending,
                            },
                            (Ok(_), ConnectedState::Handshaking, false) => {
                                *state = ConnectedState::Established;
                                Poll::Ready(Some(Event::Connected))
                            }
                            (Ok(packet), ConnectedState::Established, _) => {
                                Poll::Ready(Some(Event::Data(packet)))
                            }
                            (Err(e), _, _) => {
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
                }
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        // FIXME
        (0, None)
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
            } => transport.poll_ready_unpin(cx).map_err(|e| e.into()),
            ClientStateProject::Shutdown => Poll::Ready(Ok(())),
        }
    }

    fn start_send(mut self: Pin<&mut Self>, item: Event) -> Result<(), Self::Error> {
        let mut this = self.as_mut().project();

        if item == Event::Shutdown {
            this.inner.set(ClientState::Shutdown);
        }

        match this.inner.project() {
            ClientStateProject::Disconnected => Err(Error::Disconnected),
            ClientStateProject::Connecting(_) => Err(Error::Disconnected),
            ClientStateProject::Connected {
                ref mut transport, ..
            } => match item {
                Event::Data(packet) => transport.start_send_unpin(packet).map_err(|e| e.into()),
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
            } => transport.poll_flush_unpin(cx).map_err(|e| e.into()),
            ClientStateProject::Shutdown => Poll::Ready(Ok(())),
        }
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        match self.project().inner.project() {
            ClientStateProject::Disconnected => Poll::Ready(Ok(())),
            ClientStateProject::Connecting(_) => Poll::Pending,
            ClientStateProject::Connected {
                ref mut transport, ..
            } => transport.poll_close_unpin(cx).map_err(|e| e.into()),
            ClientStateProject::Shutdown => Poll::Ready(Ok(())),
        }
    }
}
