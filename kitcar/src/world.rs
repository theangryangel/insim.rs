//! World
//!
//! Refactored to use a WorldBuilder for a more fluent API.

use std::{
    collections::{HashMap, VecDeque},
    fmt::Debug,
    time::Duration,
};

use crate::{context::Game, Context, Engine};

/// The Workshop struct is responsible for building the Chassis
// TODO: Probably don't need this now.
#[derive(Debug)]
pub struct Workshop<S, P, C, G>
where
    S: Default + Debug,
    P: Default + Debug,
    C: Default + Debug,
    G: Default + Debug,
{
    systems: Vec<Box<dyn Engine<S, P, C, G>>>,
    state: S,
}

impl<S, P, C, G> Workshop<S, P, C, G>
where
    S: Default + Debug,
    P: Default + Debug,
    C: Default + Debug,
    G: Default + Debug,
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
    pub fn add_engine(mut self, system: impl Engine<S, P, C, G> + 'static) -> Self {
        self.systems.push(Box::new(system));
        self
    }

    /// Builds the `World` instance, consuming the builder.
    /// It connects to the network and then starts up all systems.
    pub fn ignition(
        self,
        network_builder: insim::builder::Builder,
    ) -> Result<Chassis<S, P, C, G>, insim::Error> {
        let network = network_builder.connect_blocking()?;

        Ok(Chassis {
            systems: self.systems,
            context: Context {
                stop: false,
                outgoing_packets: VecDeque::new(),
                state: self.state,
                game: Game::default(),
                connections: HashMap::new(),
                players: HashMap::new(),
            },
            network,
        })
    }
}

/// The main scheduling engine with built-in networking.
#[derive(Debug)]
pub struct Chassis<S, P, C, G>
where
    S: Default + Debug,
    P: Default + Debug,
    C: Default + Debug,
    G: Default + Debug,
{
    pub(crate) systems: Vec<Box<dyn Engine<S, P, C, G>>>,
    pub(crate) context: Context<S, P, C, G>,
    pub(crate) network: insim::net::blocking_impl::Framed,
}

impl<S, P, C, G> Chassis<S, P, C, G>
where
    S: Default + Debug,
    P: Default + Debug,
    C: Default + Debug,
    G: Default + Debug,
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
