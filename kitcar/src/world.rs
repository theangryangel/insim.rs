//! World
//!
//! Refactored to use a WorldBuilder for a more fluent API.

use std::{
    collections::{HashMap, VecDeque},
    fmt::Debug,
    time::Duration,
};

use insim::identifiers::{ConnectionId, PlayerId};

use crate::Engine;

#[derive(Debug)]
/// PlayerInfo
pub struct Player<S> {
    /// PlayerId
    pub plid: PlayerId,

    /// State
    pub state: S,
}

#[derive(Debug)]
/// ConnectionInfo
pub struct Connection<S> {
    /// ConnectionId
    pub ucid: ConnectionId,

    /// State
    pub state: S,
}

/// A container for the user-supplied state.
#[derive(Debug)]
pub struct Context<S, P, C>
where
    S: Default + Debug,
    P: Default + Debug,
    C: Default + Debug,
{
    pub(crate) stop: bool,
    pub(crate) outgoing_packets: VecDeque<insim::Packet>,

    /// State
    pub state: S,

    /// Connections list
    pub connections: HashMap<ConnectionId, Connection<C>>,

    /// Players list
    pub players: HashMap<PlayerId, Player<P>>,
}

impl<S, P, C> Context<S, P, C>
where
    S: Default + Debug,
    P: Default + Debug,
    C: Default + Debug,
{
    /// A convenience method to shutdown.
    pub fn shutdown(&mut self) {
        self.stop = true;
    }

    /// A convenience method to queue a packet for later sending.
    pub fn queue_packet<I: Into<insim::Packet>>(&mut self, packet: I) {
        self.outgoing_packets.push_back(packet.into());
    }

    fn packet(&mut self, packet: &insim::Packet) {
        match packet {
            insim::Packet::Ncn(ncn) => self.ncn(ncn),
            _ => {},
        }
    }

    fn ncn(&mut self, ncn: &insim::insim::Ncn) {
        let _ = self.connections.insert(
            ncn.ucid.clone(),
            Connection {
                ucid: ncn.ucid.clone(),
                state: C::default(),
            },
        );
    }
}

/// The Workshop struct is responsible for building the Chassis
// TODO: Probably don't need this now.
#[derive(Debug)]
pub struct Workshop<S, P, C>
where
    S: Default + Debug,
    P: Default + Debug,
    C: Default + Debug,
{
    systems: Vec<Box<dyn Engine<S, P, C>>>,
    state: S,
}

impl<S, P, C> Workshop<S, P, C>
where
    S: Default + Debug,
    P: Default + Debug,
    C: Default + Debug,
{
    /// Creates a new `Workshop`
    pub fn new(state: S) -> Self {
        Self {
            systems: Vec::new(),
            state,
        }
    }

    /// Adds a system to the `WorldBuilder`.
    /// This method takes `self` and returns `Self` to allow for method chaining.
    pub fn add_engine(mut self, system: impl Engine<S, P, C> + 'static) -> Self {
        self.systems.push(Box::new(system));
        self
    }

    /// Builds the `World` instance, consuming the builder.
    /// It connects to the network and then starts up all systems.
    pub fn ignition(
        self,
        network_builder: insim::builder::Builder,
    ) -> Result<Chassis<S, P, C>, insim::Error> {
        let network = network_builder.connect_blocking()?;

        Ok(Chassis {
            systems: self.systems,
            context: Context {
                stop: false,
                outgoing_packets: VecDeque::new(),
                state: self.state,
                connections: HashMap::new(),
                players: HashMap::new(),
            },
            network,
        })
    }
}

/// The main scheduling engine with built-in networking.
#[derive(Debug)]
pub struct Chassis<S, P, C>
where
    S: Default + Debug,
    P: Default + Debug,
    C: Default + Debug,
{
    pub(crate) systems: Vec<Box<dyn Engine<S, P, C>>>,
    pub(crate) context: Context<S, P, C>,
    pub(crate) network: insim::net::blocking_impl::Framed,
}

impl<S, P, C> Chassis<S, P, C>
where
    S: Default + Debug,
    P: Default + Debug,
    C: Default + Debug,
{
    /// Connects to the network and starts up systems.
    /// This is a private method, only called by the WorldBuilder's `build` method.
    fn startup(&mut self) {
        for system in &mut self.systems {
            system.startup(&mut self.context);
        }
    }

    /// Shuts down all systems.
    fn shutdown(&mut self) {
        for system in &mut self.systems {
            system.shutdown(&mut self.context);
        }
    }

    /// Main tick - processes network and then systems.
    pub fn tick(&mut self) {
        if let Ok(packet) = self.network.read() {
            self.context.packet(&packet);

            for system in &mut self.systems.iter_mut() {
                system.packet(&mut self.context, &packet);
            }
        }

        for system in self.systems.iter_mut() {
            system.tick(&mut self.context);
        }

        while let Some(packet) = self.context.outgoing_packets.pop_front() {
            if let Err(e) = self.network.write(packet) {
                eprintln!("Failed to send packet: {}", e);
                break;
            }
        }
    }

    /// Are we running?
    fn running(&self) -> bool {
        !self.context.stop
    }

    /// Run
    pub fn run(&mut self, tick_rate: Duration) {
        self.startup();

        // Run the game loop
        while self.running() {
            self.tick();

            // FIXME: This doesn't handle clock variation
            std::thread::sleep(tick_rate);
        }

        // Shutdown
        self.shutdown();
    }
}
