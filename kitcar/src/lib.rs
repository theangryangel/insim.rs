//! Kitcar - intended to help you make simple single server mini-games quickly using the `insim` crate.
//! By itself it does nothing.
use std::any::Any;

pub mod timers;
pub mod world;

pub use timers::Timer;
pub use world::{Chassis, Context, Workshop};

/// A marker trait for any type that can be used as a message.
pub trait Message: 'static + Send {}

/// Unified trait for all systems.
pub trait Engine<C>: 'static + Send + std::fmt::Debug {
    /// On startup handler
    fn startup(&mut self, _context: &mut Context<C>) {}
    /// On packet handler
    fn packet(&mut self, _context: &mut Context<C>, _packet: &insim::Packet) {}
    /// On shutdown handler
    fn shutdown(&mut self, _context: &mut Context<C>) {}
    /// On tick
    fn tick(&mut self, _context: &mut Context<C>) {}
    /// On inter-system message
    fn handle_message(&mut self, _context: &mut Context<C>, _message: &dyn Any) {}
}
