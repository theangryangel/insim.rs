//! Various identifiers used within insim, such as RequestId, ConnectionId, etc.

mod click;
mod connection;
mod request;

pub use click::ClickId;
pub use connection::ConnectionId;
/// Rexported from insim_core for backwards compatibility
pub use insim_core::identifiers::PlayerId;
pub use request::RequestId;
