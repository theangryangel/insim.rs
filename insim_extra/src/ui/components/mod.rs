//! Reusable pieces built on the core UI: a shared dark [`style`], and a
//! [`Dialog`] panel with injectable footer buttons.
//!
//! The core ([`Node`](super::Node), [`Screen`](super::Screen),
//! [`Component`](super::Component)) knows nothing of how things look; what's
//! here gives kitcar's panels and the examples one consistent look.

pub mod dialog;
pub mod style;

pub use dialog::{Dialog, dialog};
