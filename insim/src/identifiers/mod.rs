//! Typed identifiers used throughout the InSim protocol.
//!
//! Rather than passing raw `u8` values everywhere, this crate wraps each identifier in a
//! distinct newtype. This makes it impossible to accidentally pass a [`ConnectionId`] where
//! a [`PlayerId`] is expected - a very common source of bugs when working directly with
//! InSim.txt byte values.
//!
//! ## Which identifier do I need?
//!
//! | Type | InSim.txt name | Assigned when | Scope |
//! |---|---|---|---|
//! | [`ConnectionId`] | UCID | A client connects to LFS | Lasts until the client disconnects |
//! | [`PlayerId`] | PLID | A connection takes a car | Lasts until the car is removed |
//! | [`RequestId`] | ReqI | You set it on an outbound packet | Echoed back in the reply |
//! | [`ClickId`] | ClickID | You create a [`crate::insim::Btn`] | Lives as long as the button |
//!
//! A single connection can have many players over its lifetime (e.g. spectate → join race → pit
//! → join again). The [`PlayerId`] changes each time; the [`ConnectionId`] does not.
//! The two values are unrelated - you cannot convert between them.

mod click;
mod connection;
mod request;

pub use click::ClickId;
pub use connection::ConnectionId;
/// Rexported from insim_core for backwards compatibility
pub use insim_core::identifiers::PlayerId;
pub use request::RequestId;
