//! An optional high level API for working with LFS through Insim.
//!
//! # Example
//! ```rust
//! // Example handler usage that counts the number of packets received and resets on each
//! // reconnection.
//! struct Counter {
//!     i: AtomicUsize,
//! }
//!
//! // We must implement the EventHandler trait. You do not need to implement all the methods as
//! // the trait provides default implementations for the ones you do not need.
//! impl insim::framework::EventHandler for Counter {
//!     // The on_connection function is called when the connection is established.
//!     fn on_connect(&self, ctx: &insim::framework::Client) {
//!         // on connection reset our AtomicUsize back to 0.
//!         self.i.store(0, Ordering::Relaxed);
//!
//!         // Select a server to receive packets from.
//!         ctx.send(
//!             insim::protocol::relay::HostSelect {
//!                 hname: "Nubbins AU Demo".into(),
//!                 ..Default::default()
//!             }
//!             .into(),
//!         );
//!     }
//!
//!     // The on_raw function is called whenever any packet is received.
//!     // There are many other event functions that can be used. i.e. on_tiny to receive Tiny
//!     // packets. on_raw is provided as an escape hatch for all packets.
//!     fn on_raw(&self, ctx: &insim::framework::Client, _data: &insim::protocol::Packet) {
//!         self.i.fetch_add(1, Ordering::Relaxed);
//!     }
//! }
//!
//! // Create a Config object where we indicate that we want to use the Insim Relay,
//! // and that we want to use our Counter struct which implements the EventHandler trait.
//! let client = insim::framework::Config::default()
//!     .relay()
//!     .using_event_handler(Counter {
//!         i: AtomicUsize::new(0),
//!     })
//!     .build();
//!
//! // We instruct the client to make a connection, and then run.
//! // If you wish to run multiple futures concurrently (i.e. to run a web server in the
//! // background), you can use `tokio::select!` and a loop.
//! // See the examples directory of the crate for an example of this.
//! let res = client.run().await;
//!
//! // When run returns, we can look at the result to see what happened.
//! match res {
//!     Ok(()) => {
//!         println!("Clean shutdown");
//!     }
//!     Err(e) => {
//!         println!("Unclean shutdown: {:?}", e);
//!     }
//! }
//! ```

pub(crate) mod config;
pub(crate) mod macros;

pub use config::Config;

use super::{error, protocol};

use futures::prelude::*;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tracing;

/// A high level Client that connects to an Insim server, and handles packet event routing to
/// registered [EventHandlers](EventHandler).
pub struct Client {
    config: Arc<config::Config>,

    shutdown: Option<mpsc::UnboundedSender<bool>>,
    tx: Option<mpsc::UnboundedSender<protocol::Packet>>,
}

impl Client {
    /// Create a new Client with the given [Config].
    pub fn from_config(config: config::Config) -> Self {
        Self {
            config: Arc::new(config),
            tx: None,
            shutdown: None,
        }
    }

    /// Send a [Packet](super::protocol::Packet).
    #[allow(unused_must_use)] // if this fails then the we're probably going to die anyway
    pub fn send(&self, data: protocol::Packet) {
        if let Some(tx) = &self.tx {
            tx.send(data);
        }
    }

    /// Request shutdown of the client.
    #[allow(unused_must_use)] // if this fails then the we're probably going to die anyway
    pub fn shutdown(&self) {
        if let Some(shutdown) = &self.shutdown {
            shutdown.send(true);
        }
    }

    /// Run the client.
    /// This will not return until either the client is shutdown or an error occurs.
    pub async fn run(mut self) -> Result<(), error::Error> {
        let (shutdown_tx, mut shutdown_rx) = mpsc::unbounded_channel();
        let (send_tx, mut send_rx) = mpsc::unbounded_channel();

        self.tx = Some(send_tx);
        self.shutdown = Some(shutdown_tx);

        for event_handler in self.config.event_handlers.iter() {
            event_handler.on_startup();
        }

        let hname = &self.config.host;
        let tcp: TcpStream = TcpStream::connect(hname).await.unwrap();

        // TODO handle connection error

        let mut transport = protocol::transport::Transport::new(tcp, self.config.codec_mode);
        let isi = protocol::insim::Init {
            name: self.config.name.to_owned().into(),
            password: self.config.password.to_owned().into(),
            prefix: self.config.prefix,
            version: protocol::insim::VERSION,
            interval: self.config.interval_ms,
            flags: self.config.flags,
            reqi: 1,
        };

        let res = transport.send(isi).await;
        if let Err(e) = res {
            return Err(e.into());
        }

        // TODO handle handshake errors

        for event_handler in self.config.event_handlers.iter() {
            event_handler.on_connect(&self);
        }

        let mut ret: Result<(), error::Error> = Ok(());

        loop {
            tokio::select! {
                Some(_) = shutdown_rx.recv() => {
                    tracing::debug!("shutdown requested");
                    break;
                },

                Some(frame) = send_rx.recv() => {
                    if let Err(e) = transport.send(frame).await {
                        ret = Err(e.into());
                        break;
                    }
                }

                Some(result) = transport.next() => {

                    match result {
                        Ok(frame) => {
                            for event_handler in self.config.event_handlers.iter() {
                                event_handler.on_raw(&self, &frame);
                            }

                            self.on_packet(&frame);
                        },
                        Err(e) => {
                            ret = Err(e);
                            break;
                        }
                    }

                }
            }
        }

        for event_handler in self.config.event_handlers.iter() {
            event_handler.on_shutdown();
        }

        self.tx = None;
        self.shutdown = None;

        ret
    }
}

use crate::event_handler;
use crate::protocol::Packet;

event_handler!(
    #[allow(unused)]
    pub trait EventHandler for Client, Packet {
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

        /// Called when a [Packet::RelayError](super::protocol::Packet::RelayError) is received.
        RelayError(protocol::relay::Error) => on_relay_error,
    }
);
