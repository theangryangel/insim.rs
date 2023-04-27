use tokio::{
    sync::{mpsc, oneshot},
    task::JoinHandle,
};

use insim::{
    connection::ConnectionTrait,
    packets::{Packet, RequestIdentifiable},
};

use super::actor::{run_actor, Actor, Message};

pub struct Handle {
    sender: mpsc::Sender<Message>,
}

impl Handle {
    pub fn new<T>(connection: T) -> (Self, JoinHandle<()>)
    where
        T: ConnectionTrait + 'static,
    {
        let (sender, receiver) = mpsc::channel(16);
        let actor = Actor::new(connection, receiver);

        let handle = tokio::spawn(async move { run_actor(actor).await });

        (Self { sender }, handle)
    }

    pub async fn send_and_await_with_request_identifier<T: Into<Packet> + RequestIdentifiable>(
        &self,
        packet: T,
    ) -> Packet {
        let packet = packet.into();

        let (send, recv) = oneshot::channel();
        let msg = Message::SendAndAwaitRequest {
            packet,
            respond_to: send,
        };

        // Ignore send errors. If this send fails, so does the
        // recv.await below. There's no reason to check for the
        // same failure twice.
        let _ = self.sender.send(msg).await;
        recv.await.expect("Actor task has been killed")
    }

    pub async fn send<T: Into<Packet>>(&self, packet: T) {
        let packet = packet.into();
        let msg = Message::Send(packet);
        self.sender.send(msg).await;
    }

    pub async fn stream(&self) -> tokio::sync::broadcast::Receiver<Packet> {
        let (send, recv) = oneshot::channel();
        let msg = Message::All { respond_to: send };

        // Ignore send errors. If this send fails, so does the
        // recv.await below. There's no reason to check for the
        // same failure twice.
        let _ = self.sender.send(msg).await;
        recv.await.expect("Actor task has been killed")
    }

    pub async fn shutdown(&self) {
        self.sender.send(Message::Shutdown).await;
    }
}
