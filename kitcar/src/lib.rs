//! Kitcar - intended to help you make simple single server mini-games quickly using the `insim` crate.
//! By itself it does nothing.

#[allow(missing_docs)]
pub mod ui;

pub mod context;
pub mod framework;
pub mod plugin;
pub mod state;
pub mod time;

pub use framework::Framework;
pub use plugin::Plugin;
pub use context::Context;
