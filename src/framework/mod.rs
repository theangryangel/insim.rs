//! An optional high level API for working with LFS through Insim.
//! :warning: API is not stable.

pub(crate) mod config;
pub(crate) mod macros;

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

use std::sync::Arc;
use tokio::net::TcpStream;

use pin_project::pin_project;

#[pin_project(project = StateProj)]
enum State {
    Disconnected,

    //Connecting {
    //    inner: Box<Pin<dyn Future<Output=TcpStream>>>,
    //},
    Connected {
        #[pin]
        inner: protocol::transport::Transport<TcpStream>,
    },

    Shutdown,
}

impl ::std::fmt::Display for State {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match *self {
            State::Disconnected => write!(f, "State: Disconnected"),
            State::Connected { .. } => write!(f, "State: Connected"),
            State::Shutdown => write!(f, "State: Shutdown"),
        }
    }
}

use futures::{Sink, Stream};
use std::pin::Pin;
use std::task::{Context, Poll};

impl Stream for State {
    type Item = Event;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        match self.as_mut().project() {
            StateProj::Shutdown => Poll::Ready(None),
            StateProj::Disconnected => Poll::Pending,
            StateProj::Connected { inner, .. } => match inner.poll_next(cx) {
                Poll::Pending => Poll::Pending,
                Poll::Ready(Some(Ok(frame))) => Poll::Ready(Some(Event::Packet(frame))),
                Poll::Ready(Some(Err(e))) => Poll::Ready(Some(Event::Error(e))),
                Poll::Ready(None) => {
                    self.set(State::Disconnected);
                    Poll::Ready(Some(Event::Disconnected))
                }
            },
        }
    }
}

impl Sink<Event> for State {
    type Error = error::Error;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        match self.project() {
            StateProj::Disconnected => Poll::Pending,
            //State::Connecting{..} => { Poll::Pending },
            StateProj::Shutdown => Poll::Pending,
            StateProj::Connected { inner, .. } => match inner.poll_ready(cx) {
                Poll::Pending => Poll::Pending,
                Poll::Ready(Ok(())) => Poll::Ready(Ok(())),
                Poll::Ready(Err(e)) => Poll::Ready(Err(e.into())),
            },
        }
    }

    fn start_send(mut self: Pin<&mut Self>, value: Event) -> Result<(), Self::Error> {
        match self.as_mut().project() {
            StateProj::Disconnected => Ok(()),
            StateProj::Shutdown => Ok(()),
            //State::Connecting{..} => { Ok(()) },
            StateProj::Connected { inner, .. } => {
                match value {
                    Event::Packet(frame) => match inner.start_send(frame) {
                        Err(e) => Err(e.into()),
                        _ => Ok(()),
                    },
                    Event::Shutdown => {
                        self.set(State::Shutdown);
                        Ok(())
                    }
                    _ => {
                        // TODO: return an error
                        Ok(())
                    }
                }
            }
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        match self.project() {
            StateProj::Disconnected => Poll::Ready(Ok(())),
            StateProj::Shutdown => Poll::Ready(Ok(())),
            //State::Connecting{..} => { Poll::Ready(Ok(())) },
            StateProj::Connected { inner, .. } => match inner.poll_flush(cx) {
                Poll::Ready(Err(e)) => Poll::Ready(Err(e.into())),
                _ => Poll::Ready(Ok(())),
            },
        }
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        match self.project() {
            StateProj::Disconnected => Poll::Ready(Ok(())),
            StateProj::Shutdown => Poll::Ready(Ok(())),
            //State::Connecting{..} => { Poll::Ready(Ok(())) },
            StateProj::Connected { inner, .. } => match inner.poll_close(cx) {
                Poll::Ready(Err(e)) => Poll::Ready(Err(e.into())),
                _ => Poll::Ready(Ok(())),
            },
        }
    }
}

/// A high level Client that connects to an Insim server, and handles packet event routing to
/// registered handlers.
#[pin_project]
pub struct Client {
    pub config: Arc<config::Config>,
    #[pin]
    state: State,
    attempt: u32,
}

impl Client {
    pub fn new(config: Config) -> Self {
        Self {
            config: Arc::new(config),
            state: State::Disconnected,
            attempt: 0,
        }
    }

    pub fn shutdown(&mut self) {
        self.state = State::Shutdown;
    }
}

impl Stream for Client {
    type Item = Event;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        let mut this = self.as_mut().project();

        if let State::Disconnected = *this.state {
            // TODO: This should be moved to a future and into State(?) so that we can handle the
            // reconnection in an async manner. Currently this blocks poll_next and worse may
            // break in a tokio::select! loop, I guess?
            if *this.attempt > 0
                && (!this.config.reconnect || *this.attempt > this.config.max_reconnect_attempts)
            {
                return Poll::Ready(None);
            }

            tracing::debug!(
                "disconnected... attempting reconnect {}/{}",
                *this.attempt,
                this.config.max_reconnect_attempts
            );

            let tcp = ::std::net::TcpStream::connect(this.config.host.to_owned());
            *this.attempt += 1;

            match tcp {
                Ok(tcp) => {
                    let _ = tcp.set_nonblocking(true);
                    let inner = protocol::transport::Transport::new(
                        TcpStream::from_std(tcp).unwrap(),
                        this.config.codec_mode,
                    );
                    this.state.set(State::Connected { inner });
                    *this.attempt = 1;
                    tracing::debug!("connected.");
                    return Poll::Ready(Some(Event::Connected));
                }
                Err(e) => {
                    tracing::error!("failed to establish connection: {}", e);
                    // TODO wake after X seconds
                    // This should exponentially backoff
                    cx.waker().wake_by_ref();
                    return Poll::Pending;
                }
            }
        };

        this.state.poll_next(cx)
    }
}

impl Sink<Event> for Client {
    type Error = error::Error;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project().state.poll_ready(cx)
    }

    fn start_send(self: Pin<&mut Self>, value: Event) -> Result<(), Self::Error> {
        self.project().state.start_send(value)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project().state.poll_flush(cx)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project().state.poll_close(cx)
    }
}

//use crate::packet_handlers;
//use crate::protocol::Packet;
//use crate::protocol::transport::Transport;

/*

packet_handlers!(
    Client<State> for Packet {

        /// Called when a [Packet::Tiny](super::protocol::Packet::Tiny) is received.
        Tiny(protocol::insim::Tiny) => on_tiny,

        /// Called when a [Packet::Small](super::protocol::Packet::Small) is received.
        Small(protocol::insim::Small) => on_small,

        /// Called when a [Packet::State](super::protocol::Packet::State) is received.
        State(protocol::insim::Sta) => on_state_change,

        /// Called when a [Packet::MessageOut](super::protocol::Packet::MessageOut) is received.
        MessageOut(protocol::insim::Mso) => on_message,

        /// Called when a [Packet::VoteNotification](super::protocol::Packet::VoteNotification) is received.
        VoteNotification(protocol::insim::Vtn) => on_vote,

        /// Called when a [Packet::RaceStart](super::protocol::Packet::RaceStart) is received.
        RaceStart(protocol::insim::Rst) => on_race_start,

        /// Called when a [Packet::NewConnection](super::protocol::Packet::NewConnection) is received.
        NewConnection(protocol::insim::Ncn) => on_new_connection,

        /// Called when a [Packet::ConnectionLeave](super::protocol::Packet::ConnectionLeave) is received.
        ConnectionLeave(protocol::insim::Cnl) => on_connection_left,

        /// Called when a [Packet::NewPlayer](super::protocol::Packet::NewPlayer) is received.
        NewPlayer(protocol::insim::Npl) => on_new_player,

        /// Called when a [Packet::PlayerPits](super::protocol::Packet::PlayerPits) is received.
        PlayerPits(protocol::insim::Plp) => on_player_telepit,

        /// Called when a [Packet::PlayerLeave](super::protocol::Packet::PlayerLeave) is received.
        PlayerLeave(protocol::insim::Pll) => on_player_left,

        /// Called when a [Packet::Lap](super::protocol::Packet::Lap) is received.
        Lap(protocol::insim::Lap) => on_lap,

        /// Called when a [Packet::SplitX](super::protocol::Packet::SplitX) is received.
        SplitX(protocol::insim::Spx) => on_split,

        /// Called when a [Packet::PitStopStart](super::protocol::Packet::PitStopStart) is received.
        PitStopStart(protocol::insim::Pit) => on_pit_stop_start,

        /// Called when a [Packet::PitStopFinish](super::protocol::Packet::PitStopFinish) is received.
        PitStopFinish(protocol::insim::Psf) => on_pit_stop_finish,

        /// Called when a [Packet::PitLane](super::protocol::Packet::PitLane) is received.
        PitLane(protocol::insim::Pla) => on_player_pit_lane_change,

        /// Called when a [Packet::Penalty](super::protocol::Packet::Penalty) is received.
        Penalty(protocol::insim::Pen) => on_penalty_change,

        /// Called when a [Packet::TakeOverCar](super::protocol::Packet::TakeOverCar) is received.
        TakeOverCar(protocol::insim::Toc) => on_player_take_over,

        /// Called when a [Packet::Flag](super::protocol::Packet::Flag) is received.
        Flag(protocol::insim::Flg) => on_player_flag_change,

        /// Called when a [Packet::Finished](super::protocol::Packet::Finished) is received.
        Finished(protocol::insim::Fin) => on_player_race_finish,

        /// Called when a [Packet::Result](super::protocol::Packet::Result) is received.
        Result(protocol::insim::Res) => on_player_race_result,

        /// Called when a [Packet::NodeLap](super::protocol::Packet::NodeLap) is received.
        NodeLap(protocol::insim::Nlp) => on_node_info,

        /// Called when a [Packet::MutliCarInfo](super::protocol::Packet::MultiCarInfo) is received.
        MultiCarInfo(protocol::insim::Mci) => on_multi_car_info,

        /// Called when a [Packet::CarReset](super::protocol::Packet::CarReset) is received.
        CarReset(protocol::insim::Crs) => on_player_vehicle_reset,

        /// Called when a [Packet::Contact](super::protocol::Packet::Contact) is received.
        Contact(protocol::insim::Con) => on_player_contact,

        /// Called when a [Packet::ObjectHit](super::protocol::Packet::ObjectHit) is received.
        ObjectHit(protocol::insim::Obh) => on_player_object_hit,

        /// Called when a [Packet::HotLapValidity](super::protocol::Packet::HotLapValidity) is received.
        HotLapValidity(protocol::insim::Hlv) => on_player_hot_lap_validity_failure,

        /// Called when a [Packet::RelayHostList](super::protocol::Packet::RelayHostList) is received.
        RelayHostList(protocol::relay::HostList) => on_relay_host_list,

        /// Called when a [Packet::RelayError](super::protocol::Packet::RelayError) is received.
        RelayError(protocol::relay::Error) => on_relay_error,
    }
);

*/
