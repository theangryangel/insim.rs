use crate::{codec::VersionedFrame, error::Error};

use super::ConnectionIdentifier;

#[derive(Debug, Clone)]
/// Events which can be yielded by [super::Connection] poll method
pub enum Event<P: VersionedFrame> {
    Connected(Option<ConnectionIdentifier>),
    Disconnected(Option<ConnectionIdentifier>),
    Data(P, Option<ConnectionIdentifier>),
    Error(Error, Option<ConnectionIdentifier>),
    Shutdown(Option<ConnectionIdentifier>),
}
