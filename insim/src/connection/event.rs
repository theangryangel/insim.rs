use crate::{error::Error, packets::Packet};

#[derive(Debug, Clone)]
pub enum Event {
    Connected,
    Disconnected,
    Data(Packet),
    Error(Error),
}
