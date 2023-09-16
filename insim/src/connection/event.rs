use crate::{error::Error, codec::{Codec, Packets}};

use super::ConnectionIdentifier;

#[derive(Debug, Clone)]
/// Events which can be yielded by [super::Connection] poll method
pub enum Event<P: Packets> {
    Connected(Option<ConnectionIdentifier>),
    Disconnected(Option<ConnectionIdentifier>),
    Data(P, Option<ConnectionIdentifier>),
    Error(Error, Option<ConnectionIdentifier>),
    Shutdown(Option<ConnectionIdentifier>),
}
