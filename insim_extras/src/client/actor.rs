use std::collections::HashMap;

use insim::{
    core::identifiers::{ConnectionId, PlayerId, RequestId},
    packets::Packet,
    packets::RequestIdentifiable,
    prelude::*,
};
use tokio::sync::{broadcast, mpsc, oneshot};
use tracing;

use crate::game_state::{Connection, GameState};

pub(crate) struct Actor<T>
where
    T: ConnectionTrait,
{
    receiver: mpsc::Receiver<Message>,
    connection: T,
    state: GameState,
    broadcast: broadcast::Sender<Packet>,
    next_request_identifier: u8,
    pending_with_request_identifier: HashMap<u8, oneshot::Sender<Packet>>,
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

    Connection {
        respond_to: oneshot::Sender<Connection>,
        ucid: ConnectionId,
    },

    ConnectionList {
        players_only: bool,
        respond_to: oneshot::Sender<Vec<Connection>>,
    },

    Player {
        respond_to: oneshot::Sender<Option<Connection>>,
        ucid: PlayerId,
    },
}

impl<T> Actor<T>
where
    T: ConnectionTrait,
{
    pub(crate) fn new(connection: T, receiver: mpsc::Receiver<Message>) -> Self {
        let (broadcast, _) = broadcast::channel(25);
        Actor {
            receiver,
            next_request_identifier: 5,
            connection,
            broadcast,
            pending_with_request_identifier: HashMap::new(),
            state: GameState::new(),
        }
    }

    fn next_id(&mut self) -> u8 {
        let id = self.next_request_identifier.wrapping_add(1);
        self.next_request_identifier = id;
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

                self.pending_with_request_identifier.insert(id, respond_to);

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

            Message::ConnectionList {
                players_only: true,
                respond_to,
            } => {
                let _ = respond_to.send(self.state.players());
            }

            Message::ConnectionList {
                players_only: false,
                respond_to,
            } => {
                let _ = respond_to.send(self.state.connections());
            }

            Message::Player { respond_to, plid } => {
                let _ = respond_to.send(self.state.player(plid));
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
                    actor.state.handle_packet(&packet);

                    if actor.broadcast.receiver_count() > 0 {
                        actor.broadcast.send(packet.clone()).unwrap();
                    }

                    if let Some(a) = actor.pending_with_request_identifier.remove(&packet.request_identifier()) {
                        a.send(packet);
                    }
                }
            },

        }
    }
}
