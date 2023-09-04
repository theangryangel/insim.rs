use crate::{error::Error, packets::Packet};

#[derive(Debug, Clone)]
/// Events which can be yielded by [super::Connection] poll method
pub enum Event {
    Connected,
    Disconnected,
    Data(Packet),
    Error(Error),
    Shutdown,
}
