use crate::{error::Error, packet::Packet};

use super::ConnectionIdentifier;

#[derive(Debug, Clone)]
/// Events which can be yielded by [super::Connection] poll method
pub enum Event {
    Connected(Option<ConnectionIdentifier>),
    Disconnected(Option<ConnectionIdentifier>),
    Data(Packet, Option<ConnectionIdentifier>),
    Error(Error, Option<ConnectionIdentifier>),
    Shutdown(Option<ConnectionIdentifier>),
}
