use super::event::Event;
use crate::packets::Packet;

use tokio::sync::{broadcast, oneshot};

pub(crate) enum Command {
    Send(Packet),
    Firehose(oneshot::Sender<broadcast::Receiver<Event>>),
    Shutdown,
}
