//! World
//!
//! Refactored to use a WorldBuilder for a more fluent API.

use std::{any::Any, collections::VecDeque, time::Duration};

use insim::Packet;

use crate::{Engine, Message};

/// A container for the user-supplied state.
#[derive(Debug)]
pub struct Context<C> {
    /// User-supplied state
    pub state: C,
    pub(crate) stop: bool,
    pub(crate) outgoing_packets: VecDeque<insim::Packet>,
    pub(crate) mailbox: VecDeque<Box<dyn Any + Send>>,
}

impl<C: 'static + Send> Context<C> {
    /// A convenience method to shutdown.
    pub fn shutdown(&mut self) {
        self.stop = true;
    }

    /// A convenience method for systems to send messages.
    pub fn send_message<M: Message>(&mut self, msg: M) {
        self.mailbox.push_back(Box::new(msg));
    }

    /// A convenience method to queue a packet for later sending.
    pub fn queue_packet<I: Into<insim::Packet>>(&mut self, packet: I) {
        self.outgoing_packets.push_back(packet.into());
    }
}

/// The Workshop struct is responsible for building the Chassis
#[derive(Debug)]
pub struct Workshop<C> {
    systems: Vec<Box<dyn Engine<C>>>,
    state: C,
}

impl Workshop<()> {
    /// Creates a new `WorldBuilder` with a default `()` state.
    pub fn new() -> Self {
        Self {
            systems: Vec::new(),
            state: (),
        }
    }

    /// Creates a new `WorldBuilder` specifying the user-supplied state.
    pub fn with_state<S: 'static + Send>(state: S) -> Workshop<S> {
        Workshop {
            // This is the key change: we create a new, empty systems vector
            // with the correct generic type `S`.
            systems: Vec::new(),
            state,
        }
    }
}

impl<C: 'static + Send> Workshop<C> {
    /// Adds a system to the `WorldBuilder`.
    /// This method takes `self` and returns `Self` to allow for method chaining.
    pub fn add_engine(mut self, system: impl Engine<C> + 'static) -> Self {
        self.systems.push(Box::new(system));
        self
    }

    /// Builds the `World` instance, consuming the builder.
    /// It connects to the network and then starts up all systems.
    pub fn ignition(self, network_builder: insim::builder::Builder) -> Chassis<C> {
        // FIXME: unwrap
        let network = network_builder.connect_blocking().unwrap();

        Chassis {
            systems: self.systems,
            context: Context {
                state: self.state,
                stop: false,
                outgoing_packets: VecDeque::new(),
                mailbox: VecDeque::new(),
            },
            network,
        }
    }
}

/// The main scheduling engine with built-in networking.
#[derive(Debug)]
pub struct Chassis<C> {
    pub(crate) systems: Vec<Box<dyn Engine<C>>>,
    pub(crate) context: Context<C>,
    pub(crate) network: insim::net::blocking_impl::Framed,
}

impl<C: 'static + Send> Chassis<C> {
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
            for system in &mut self.systems {
                system.packet(&mut self.context, &packet);
            }
        }

        while let Some(packet) = self.context.outgoing_packets.pop_front() {
            if let Err(e) = self.network.write(packet) {
                eprintln!("Failed to send packet: {}", e);
                break;
            }
        }

        for system in &mut self.systems {
            system.tick(&mut self.context);
        }

        let messages_to_process: VecDeque<Box<dyn Any + Send>> =
            self.context.mailbox.drain(..).collect();
        for msg in messages_to_process.iter() {
            for system in &mut self.systems {
                system.handle_message(&mut self.context, msg.as_ref());
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
