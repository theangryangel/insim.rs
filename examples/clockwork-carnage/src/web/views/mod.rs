//! Maud views — HTML as type-checked Rust functions.

pub mod components;
pub mod forms;
pub mod layout;

mod about;
mod event_detail;
mod event_edit;
mod event_new;
mod events;
mod index;
mod profile;
mod results;

pub use about::about;
pub use event_detail::event_detail;
pub use event_edit::{event_edit, event_edit_form};
pub use event_new::{event_new, event_new_form};
pub use events::{Filters, events};
pub use index::index;
pub use profile::profile;
pub use results::event_results;
