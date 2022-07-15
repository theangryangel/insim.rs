pub(crate) mod chat;
pub(crate) mod connection;
pub(crate) mod player;

use bounded_vec_deque::BoundedVecDeque;
use insim::client::prelude::*;
use miette::{IntoDiagnostic, Result};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use tokio::sync::{broadcast, mpsc};

type ConnectionMap = HashMap<u8, connection::Connection>;
type PlayerMap = HashMap<u8, player::Player>;
type ConnectionPlayerMap = HashMap<u8, u8>;
type ChatHistory = BoundedVecDeque<chat::Chat>;

#[derive(Debug, Clone)]
pub struct State(Arc<Mutex<StateInner>>);

impl State {
    pub fn new(tx: mpsc::UnboundedSender<Event>) -> Self {
        let inner = StateInner::new(tx);
        Self(Arc::new(Mutex::new(inner)))
    }

    pub fn handle_event(&self, event: Event) -> Result<()> {
        // FIXME, no unwraps plz.
        self.0.lock().unwrap().handle_event(event)
    }
}

#[derive(Debug)]
pub(crate) struct StateInner {
    pub connections: ConnectionMap,
    pub players: PlayerMap,

    pub idx_player_connection: ConnectionPlayerMap,
    pub idx_connection_player: ConnectionPlayerMap,

    pub chat: ChatHistory,

    pub tx: mpsc::UnboundedSender<Event>,
}

impl StateInner {
    pub fn new(tx: mpsc::UnboundedSender<Event>) -> Self {
        Self {
            connections: HashMap::new(),
            players: HashMap::new(),
            chat: BoundedVecDeque::new(256),
            tx: tx,
            idx_player_connection: HashMap::new(),
            idx_connection_player: HashMap::new(),
        }
    }

    pub fn handle_event(&mut self, event: Event) -> Result<()> {
        match event {
            Event::Connected => {
                self.tx
                    .send(Event::Data(Packet::Tiny(insim::protocol::insim::Tiny {
                        subtype: insim::protocol::insim::TinyType::Ncn,
                        ..Default::default()
                    })))
                    .into_diagnostic()?;

                self.tx
                    .send(Event::Data(Packet::Tiny(insim::protocol::insim::Tiny {
                        subtype: insim::protocol::insim::TinyType::Npl,
                        ..Default::default()
                    })))
                    .into_diagnostic()?;
            }

            Event::Data(Packet::MessageOut(data)) => {
                self.chat
                    .push_back(chat::Chat::new(data.ucid, data.msg.to_lossy_string()));
            }

            Event::Data(Packet::NewConnection(data)) => {
                self.connections.insert(data.ucid, (&data).into());
                self.chat.push_back(chat::Chat::new(
                    data.ucid,
                    format!("New player joined: {}", &data.uname),
                ));
            }

            Event::Data(Packet::ConnectionLeave(data)) => {
                self.connections.remove(&data.ucid);
                if let Some(plid) = self.idx_connection_player.remove(&data.ucid) {
                    self.players.remove(&plid);
                    self.idx_player_connection.remove(&plid);
                }
            }

            Event::Data(Packet::NewPlayer(data)) => {
                self.players.insert(data.plid, (&data).into());
                self.idx_player_connection.insert(data.plid, data.ucid);
                self.idx_connection_player.insert(data.ucid, data.plid);
            }

            Event::Data(Packet::PlayerLeave(data)) => {
                self.connections.remove(&data.plid);
                if let Some(ucid) = self.idx_player_connection.remove(&data.plid) {
                    self.idx_player_connection.remove(&ucid);
                }
            }

            Event::Data(Packet::MultiCarInfo(data)) => {
                for info in data.info.iter() {
                    if let Some(player) = self.players.get_mut(&info.plid) {
                        player.xyz = info.xyz.clone();
                    }
                }
            }

            _ => {}
        };

        println!("{:?}", self);

        Ok(())
    }
}
