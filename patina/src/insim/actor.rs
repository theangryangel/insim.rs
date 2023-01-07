use futures::{SinkExt, StreamExt};
use insim::{
    client::{Client, Config, Event},
    protocol::Packet,
};
use miette::Result;
use tokio::sync::{mpsc, oneshot};

use crate::state::{chat::Chat, Connection, Game, Notifiers, State};

pub(crate) enum InsimActorMessage {
    Connections {
        players_only: bool,
        flipped: bool,
        respond_to: oneshot::Sender<Vec<Connection>>,
    },

    Game {
        respond_to: oneshot::Sender<Game>,
    },

    Chat {
        respond_to: oneshot::Sender<Vec<Chat>>,
    },

    Notifiers {
        respond_to: oneshot::Sender<Notifiers>,
    },
}

pub(crate) struct InsimActor {
    rx: mpsc::Receiver<InsimActorMessage>,

    client: Client,
    state: State,
}

impl InsimActor {
    pub(crate) fn new(rx: mpsc::Receiver<InsimActorMessage>, config: Config) -> Self {
        let client = config.into_client();
        let state = State::new();

        Self { rx, client, state }
    }

    async fn handle_insim_event(&mut self, msg: &Event) -> Result<()> {
        self.state.handle_insim_event(msg)?;

        match msg {
            Event::Handshaking => {
                // TODO
            }
            Event::Connected => {
                self.client
                    .send(Event::Data(Packet::Tiny(insim::protocol::insim::Tiny {
                        subtype: insim::protocol::insim::TinyType::Ncn,
                        ..Default::default()
                    })))
                    .await?;

                self.client
                    .send(Event::Data(Packet::Tiny(insim::protocol::insim::Tiny {
                        subtype: insim::protocol::insim::TinyType::Npl,
                        ..Default::default()
                    })))
                    .await?;

                self.client
                    .send(Event::Data(Packet::Tiny(insim::protocol::insim::Tiny {
                        subtype: insim::protocol::insim::TinyType::Sst,
                        ..Default::default()
                    })))
                    .await?;
            }
            Event::Disconnected => {
                // TODO
            }
            Event::Data(_) => {
                // TODO
            }
            Event::Error(_) => {
                // TODO
            }
            Event::Shutdown => unimplemented!(),
        }

        Ok(())
    }

    fn handle_actor_message(&mut self, msg: InsimActorMessage) {
        match msg {
            InsimActorMessage::Connections {
                players_only,
                flipped,
                respond_to,
            } => {
                let connections = if players_only {
                    self.state.get_players(flipped)
                } else {
                    self.state.get_connections()
                };

                let _ = respond_to.send(connections);
            }

            InsimActorMessage::Game { respond_to } => {
                let _ = respond_to.send(self.state.game());
            }

            InsimActorMessage::Chat { respond_to } => {
                #[allow(clippy::map_clone)]
                let _ = respond_to.send(self.state.chat().iter().map(|c| c.clone()).collect());
            }

            InsimActorMessage::Notifiers { respond_to } => {
                let _ = respond_to.send(self.state.notifiers.clone());
            }
        }
    }
}

pub(crate) async fn run(mut actor: InsimActor) -> Result<()> {
    loop {
        tokio::select! {
            msg = actor.client.next() => match msg {
                Some(msg) => {
                    actor.handle_insim_event(&msg).await?;
                },
                None => {
                    break;
                }
            },

            msg = actor.rx.recv() => match msg {
                Some(msg) => actor.handle_actor_message(msg),
                None => {
                    actor.client.shutdown();
                    break;
                }
            },
        }
    }

    Ok(())
}
