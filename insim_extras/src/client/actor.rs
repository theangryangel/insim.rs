use std::collections::HashMap;

use insim::{
    core::identifiers::RequestId, packets::Packet, packets::RequestIdentifiable, prelude::*,
};
use tokio::sync::{broadcast, mpsc, oneshot};
use tracing;

pub(crate) struct Actor<T>
where
    T: ConnectionTrait,
{
    receiver: mpsc::Receiver<Message>,
    connection: T,
    next_id: u8,

    broadcast: broadcast::Sender<Packet>,

    with_reqi: HashMap<u8, oneshot::Sender<Packet>>,
}

pub(crate) enum Message {
    All {
        respond_to: oneshot::Sender<tokio::sync::broadcast::Receiver<Packet>>,
    },

    Send(Packet),

    SendAndAwaitRequest {
        packet: Packet,
        respond_to: oneshot::Sender<Packet>,
    },

    Shutdown,
}

impl<T> Actor<T>
where
    T: ConnectionTrait,
{
    pub(crate) fn new(connection: T, receiver: mpsc::Receiver<Message>) -> Self {
        let (broadcast, _) = broadcast::channel(25);
        Actor {
            receiver,
            next_id: 5,
            connection,
            broadcast,
            with_reqi: HashMap::new(),
        }
    }

    fn next_id(&mut self) -> u8 {
        let id = self.next_id.wrapping_add(1);
        self.next_id = id;
        id
    }

    async fn handle_message(&mut self, msg: Message) {
        match msg {
            Message::Send(packet) => {
                self.connection.send(packet).await;
            }

            Message::SendAndAwaitRequest {
                mut packet,
                respond_to,
            } => {
                let id = self.next_id();

                self.with_reqi.insert(id, respond_to);

                RequestIdentifiable::set_request_identifier(&mut packet, RequestId(id));

                // TODO write reqi
                self.connection.send(packet).await;
            }

            Message::All { respond_to } => {
                let _ = respond_to.send(self.broadcast.subscribe());
            }

            Message::Shutdown => {
                self.connection.shutdown();
            }
        }
    }
}

pub(crate) async fn run_actor<T>(mut actor: Actor<T>)
where
    T: ConnectionTrait,
{
    loop {
        tokio::select! {
            msg = actor.receiver.recv() => match msg {
                Some(msg) => actor.handle_message(msg).await,
                None => break,
            },

            packet = actor.connection.next() => match packet {
                None => break,
                Some(Err(e)) => {
                    tracing::error!("{:?}", e);
                    break;
                },
                Some(Ok(packet)) => {
                    if actor.broadcast.receiver_count() > 0 {
                        actor.broadcast.send(packet.clone()).unwrap();
                    }

                    if let Some(a) = actor.with_reqi.remove(&packet.request_identifier()) {
                        a.send(packet);
                    }
                }
            },

        }
    }
}
