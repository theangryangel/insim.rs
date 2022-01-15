//! An optional high level API for working with LFS through Insim.
//! :warning: API is not stable.

pub(crate) mod config;

pub use config::Config;

use super::{error, protocol};

// TODO: Split this into Event and Commands
#[derive(Debug)]
pub enum Event {
    Connected,
    Disconnected,
    Shutdown,
    Packet(protocol::Packet),
    Error(error::Error),
}

use futures::{FutureExt, SinkExt, StreamExt};
use pin_project::pin_project;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::time;

#[pin_project(project = StateProj)]
enum State {
    Disconnected {
        deadline: Option<Pin<Box<time::Sleep>>>,
    },

    // Connecting {
    //     inner: Box<Pin<dyn Future<Output=TcpStream>>>,
    // },

    // Handshake {
    //     #[pin]
    //     inner: protocol::transport::Transport<TcpStream>,
    // },
    Connected {
        #[pin]
        inner: protocol::transport::Transport<TcpStream>,
    },

    Shutdown,
}

impl ::std::fmt::Display for State {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match *self {
            State::Disconnected { .. } => write!(f, "State: Disconnected"),
            // State::Connecting { .. } => write!(f, "State: Connecting..."),
            // State::Handshake { .. } => write!(f, "State: Awaiting Version Handshake..."),
            State::Connected { .. } => write!(f, "State: Connected"),
            State::Shutdown => write!(f, "State: Shutdown"),
        }
    }
}

use futures::{Sink, Stream};
use std::pin::Pin;
use std::task::{Context, Poll};

/// A high level Client that connects to an Insim server, and transparently handles reconnection
/// attempts.
#[pin_project]
pub struct Client {
    pub config: Arc<config::Config>,
    #[pin]
    state: State,
    attempt: i64,
}

impl Client {
    pub fn new(config: Config) -> Self {
        Self {
            config: Arc::new(config),
            state: State::Disconnected { deadline: None },
            attempt: 0,
        }
    }

    pub fn shutdown(&mut self) {
        self.state = State::Shutdown;
    }

    fn poll_backoff(&mut self, cx: &mut Context) -> bool {
        if self.attempt > 0
            && (!self.config.reconnect
                || (self.config.max_reconnect_attempts > 0
                    && self.attempt > self.config.max_reconnect_attempts))
        {
            tracing::debug!("skipping reconnect, max attempts reached!");
            self.state = State::Shutdown;
            cx.waker().wake_by_ref();
            return false;
        }

        let duration = time::Duration::new((self.attempt * 2).try_into().unwrap(), 0);
        let next = time::Instant::now() + duration;
        tracing::debug!("backing off until {:?}", next);
        let deadline = Box::pin(tokio::time::sleep_until(next));
        self.state = State::Disconnected {
            deadline: Some(deadline),
        };
        cx.waker().wake_by_ref();
        true
    }

    fn poll_connect(&mut self, cx: &mut Context) -> Poll<Option<Event>> {
        if self.config.reconnect {
            tracing::debug!(
                "attempting connect {}/{}",
                self.attempt,
                self.config.max_reconnect_attempts
            );
        } else {
            tracing::debug!("attempting connect");
        }

        let tcp = ::std::net::TcpStream::connect(self.config.host.to_owned());
        self.attempt += 1;

        match tcp {
            Ok(tcp) => {
                let _ = tcp.set_nonblocking(true);
                let inner = protocol::transport::Transport::new(
                    TcpStream::from_std(tcp).unwrap(),
                    self.config.codec_mode,
                );
                self.attempt = 1;

                self.state = State::Connected { inner };
                tracing::debug!("connected.");
                if self.config.verify_version {
                    unimplemented!()
                }
                Poll::Ready(Some(Event::Connected))
            }
            Err(e) => {
                tracing::error!("failed to establish connection: {}", e);
                self.poll_backoff(cx);
                Poll::Pending
            }
        }
    }
}

impl Stream for Client {
    type Item = Event;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        let this = self.as_mut().project();

        match this.state.project() {
            StateProj::Disconnected { deadline: None } => self.poll_connect(cx),

            StateProj::Disconnected {
                deadline: Some(ref mut deadline),
            } => {
                match deadline.poll_unpin(cx) {
                    Poll::Ready(t) => t,
                    Poll::Pending => return Poll::Pending,
                };

                self.poll_connect(cx)
            }

            // State::Connecting { inner } => {
            //     unimplemented!()
            // },

            // State::Handshake { inner } => {
            //     unimplemented!()
            // },
            StateProj::Connected { mut inner } => match inner.poll_next_unpin(cx) {
                Poll::Pending => Poll::Pending,
                Poll::Ready(Some(Ok(frame))) => Poll::Ready(Some(Event::Packet(frame))),
                Poll::Ready(Some(Err(e))) => match e {
                    error::Error::Timeout => {
                        self.as_mut().poll_backoff(cx);
                        Poll::Ready(Some(Event::Disconnected))
                    }
                    e => {
                        tracing::debug!("unhandled error {:?}", e);
                        Poll::Ready(Some(Event::Error(e)))
                    }
                },
                Poll::Ready(None) => {
                    self.as_mut().poll_backoff(cx);
                    Poll::Ready(Some(Event::Disconnected))
                }
            },

            StateProj::Shutdown => Poll::Ready(None),
        }
    }
}

impl Sink<Event> for Client {
    type Error = error::Error;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let this = self.project();

        match this.state.project() {
            StateProj::Disconnected { .. } => Poll::Pending,

            // State::Connecting { inner } => {
            //     unimplemented!()
            // },

            // State::Handshake { inner } => {
            //     unimplemented!()
            // },
            StateProj::Connected { ref mut inner, .. } => match inner.poll_ready_unpin(cx) {
                Poll::Pending => Poll::Pending,
                Poll::Ready(Ok(())) => Poll::Ready(Ok(())),
                Poll::Ready(Err(e)) => Poll::Ready(Err(e.into())),
            },

            StateProj::Shutdown => Poll::Ready(Err(error::Error::Shutdown)),
        }
    }

    fn start_send(self: Pin<&mut Self>, value: Event) -> Result<(), Self::Error> {
        let mut this = self.project();

        match this.state.as_mut().project() {
            StateProj::Disconnected { .. } => Err(error::Error::Disconnected),

            // State::Connecting { inner } => {
            //     unimplemented!()
            // },

            // State::Handshake { inner } => {
            //     unimplemented!()
            // },
            StateProj::Connected { ref mut inner, .. } => match value {
                Event::Packet(frame) => match inner.start_send_unpin(frame) {
                    Err(e) => Err(e.into()),
                    _ => Ok(()),
                },
                Event::Shutdown => {
                    this.state.set(State::Shutdown);
                    Ok(())
                }
                _ => Err(error::Error::Unimplemented),
            },

            StateProj::Shutdown => Err(error::Error::Disconnected),
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let this = self.project();

        match this.state.project() {
            StateProj::Disconnected { .. } => Poll::Pending,

            // State::Connecting { inner } => {
            //     unimplemented!()
            // },

            // State::Handshake { inner } => {
            //     unimplemented!()
            // },
            StateProj::Connected { ref mut inner, .. } => match inner.poll_flush_unpin(cx) {
                Poll::Pending => Poll::Pending,
                Poll::Ready(Err(e)) => Poll::Ready(Err(e.into())),
                Poll::Ready(Ok(())) => Poll::Ready(Ok(())),
            },

            StateProj::Shutdown => Poll::Ready(Err(error::Error::Shutdown)),
        }
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let this = self.project();

        match this.state.project() {
            StateProj::Disconnected { .. } => Poll::Pending,

            // State::Connecting { inner } => {
            //     unimplemented!()
            // },

            // State::Handshake { inner } => {
            //     unimplemented!()
            // },
            StateProj::Connected { ref mut inner, .. } => match inner.poll_close_unpin(cx) {
                Poll::Pending => Poll::Pending,
                Poll::Ready(Err(e)) => Poll::Ready(Err(e.into())),
                Poll::Ready(Ok(())) => Poll::Ready(Ok(())),
            },

            StateProj::Shutdown => Poll::Ready(Err(error::Error::Shutdown)),
        }
    }
}
