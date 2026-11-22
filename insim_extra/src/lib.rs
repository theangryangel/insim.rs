//! Standalone/Reusable Insim components.

// Lets `#[derive(ChatCommand)]`, which names `::insim_extra`, work in this crate.
extern crate self as insim_extra;

pub mod chat;
pub mod kitcar;
#[allow(missing_docs)]
pub mod ui;
pub mod world;
