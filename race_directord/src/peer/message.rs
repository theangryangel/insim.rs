use insim::{codec::Frame, connection::Event};
use tokio::sync::{oneshot, broadcast};


pub enum Message<T: Frame> {
    Subscribe {
        respond_to: oneshot::Sender<broadcast::Receiver<Event<T>>>
    },

    Shutdown,
}
