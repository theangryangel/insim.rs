//! Various identifiers used within insim, such as RequestId, ConnectionId, etc.

mod click;
mod connection;
mod player;
mod request;

pub use click::ClickId;
pub use connection::ConnectionId;
pub use player::PlayerId;
pub use request::RequestId;
