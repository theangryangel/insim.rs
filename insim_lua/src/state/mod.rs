pub(crate) mod chat;
pub(crate) mod connection;
pub(crate) mod player;

use bounded_vec_deque::BoundedVecDeque;
use insim::client::prelude::*;
use miette::{IntoDiagnostic, Result};
use std::collections::HashMap;
use tokio::sync::{mpsc, Notify};
use parking_lot::RwLock;

type ConnectionMap = HashMap<u8, connection::Connection>;
type PlayerMap = HashMap<u8, player::Player>;
type ConnectionPlayerMap = HashMap<u8, u8>;
type ChatHistory = BoundedVecDeque<chat::Chat>;

#[derive(Debug)]
pub struct State {
    connections: RwLock<ConnectionMap>,
    players: RwLock<PlayerMap>,

    idx_player_connection: RwLock<ConnectionPlayerMap>,
    idx_connection_player: RwLock<ConnectionPlayerMap>,

    chat: RwLock<ChatHistory>,

    tx: mpsc::UnboundedSender<Event>,

    pub change:
}

impl State {
    pub fn new(tx: mpsc::UnboundedSender<Event>) -> Self {
        Self {
            connections: RwLock::new(HashMap::new()),
            players: RwLock::new(HashMap::new()),
            idx_player_connection: RwLock::new(HashMap::new()),
            idx_connection_player: RwLock::new(HashMap::new()),
            chat: RwLock::new(BoundedVecDeque::new(256)),
            tx,
            change: broadcast::channel(10).0,
        }
    }

    pub fn connections(&self) -> parking_lot::lock_api::RwLockReadGuard<'_, parking_lot::RawRwLock, ConnectionMap> {
        self.connections.read()
    }

    pub fn players(&self) -> parking_lot::lock_api::RwLockReadGuard<'_, parking_lot::RawRwLock, PlayerMap> {
        self.players.read()
    }

    pub fn chat(&self) -> parking_lot::lock_api::RwLockReadGuard<'_, parking_lot::RawRwLock, ChatHistory> {
        self.chat.read()
    }

    pub(crate) fn handle_data(&self, data: Packet) -> Result<()> {
        match data {
            Packet::MessageOut(data) => {
                self.chat
                    .write()
                    .push_back(chat::Chat::new(data.ucid, data.msg.to_lossy_string()));
            },

            Packet::NewConnection(data) => {
                self.connections.write().insert(data.ucid, (&data).into());
                self.chat.write().push_back(chat::Chat::new(
                    data.ucid,
                    format!("New player joined: {}", &data.uname),
                ));
            },

            Packet::ConnectionLeave(data) => {
                self.connections.write().remove(&data.ucid);
                if let Some(plid) = self.idx_connection_player.write().remove(&data.ucid) {
                    self.players.write().remove(&plid);
                    self.idx_player_connection.write().remove(&plid);
                }
            },

            Packet::NewPlayer(data) => {
                self.players.write().insert(data.plid, (&data).into());
                self.idx_player_connection.write().insert(data.plid, data.ucid);
                self.idx_connection_player.write().insert(data.ucid, data.plid);
            },

            Packet::PlayerLeave(data) => {
                self.players.write().remove(&data.plid);
                if let Some(ucid) = self.idx_player_connection.write().remove(&data.plid) {
                    self.idx_player_connection.write().remove(&ucid);
                }
            },

            Packet::PlayerPits(data) => {
                // Telepits
                self.players.write().remove(&data.plid);
                self.idx_player_connection.write().remove(&data.plid);
            },

            Packet::TakeOverCar(data) => {
                self.idx_player_connection.write().remove(&data.plid);
                self.idx_connection_player.write().remove(&data.olducid);

                self.idx_player_connection.write().insert(data.plid, data.newucid);
                self.idx_connection_player.write().insert(data.newucid, data.plid);
            },

            Packet::PitLane(data) => {
                if let Some(player) = self.players.write().get_mut(&data.plid) {
                    player.in_pitlane = data.entered();
                }
            },

            Packet::MultiCarInfo(data) => {
                let mut players = self.players.write();
                for info in data.info.iter() {
                    if let Some(player) = players.get_mut(&info.plid) {
                        player.xyz = info.xyz.clone();
                    }
                }
            },

            _ => {}
        }

        Ok(())
    }

    pub(crate) fn handle_event(&self, event: Event) -> Result<()> {
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
            },

            Event::Data(packet) => {
                self.handle_data(packet)?;
                //self.change.send(()).into_diagnostic()?;
            },


            _ => {}
        };

        println!("{:?}", self);

        Ok(())
    }
}
