pub(crate) mod chat;
pub(crate) mod connection;
pub(crate) mod player;

use bounded_vec_deque::BoundedVecDeque;
use insim::protocol::identifiers::ConnectionId;
use insim::{client::prelude::*, protocol::identifiers::PlayerId};
use miette::{IntoDiagnostic, Result};
use parking_lot::RwLock;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{mpsc, Notify};

type ConnectionMap = HashMap<ConnectionId, connection::Connection>;
type PlayerMap = HashMap<PlayerId, player::Player>;
type ChatHistory = BoundedVecDeque<chat::Chat>;

#[derive(Debug)]
pub struct State {
    connections: RwLock<ConnectionMap>,
    connections_notify: Arc<Notify>,

    players: RwLock<PlayerMap>,
    players_notify: Arc<Notify>,

    idx_player_connection: RwLock<HashMap<PlayerId, ConnectionId>>,
    idx_connection_player: RwLock<HashMap<ConnectionId, PlayerId>>,

    chat: RwLock<ChatHistory>,
    chat_notify: Arc<Notify>,

    tx: mpsc::UnboundedSender<Event>,
}

#[allow(unused)]
impl State {
    pub fn new(tx: mpsc::UnboundedSender<Event>) -> Self {
        Self {
            connections: RwLock::new(HashMap::new()),
            players: RwLock::new(HashMap::new()),
            idx_player_connection: RwLock::new(HashMap::new()),
            idx_connection_player: RwLock::new(HashMap::new()),
            chat: RwLock::new(BoundedVecDeque::new(256)),
            tx,
            chat_notify: Arc::new(Notify::new()),
            connections_notify: Arc::new(Notify::new()),
            players_notify: Arc::new(Notify::new()),
        }
    }

    pub fn get_connections(
        &self,
    ) -> parking_lot::lock_api::RwLockReadGuard<'_, parking_lot::RawRwLock, ConnectionMap> {
        self.connections.read()
    }

    pub fn notify_on_connection(&self) -> Arc<Notify> {
        self.connections_notify.clone()
    }

    pub fn get_connection_player_id(&self, ucid: ConnectionId) -> Option<PlayerId> {
        self.idx_connection_player.read().get(&ucid).copied()
    }

    pub fn get_players(
        &self,
    ) -> parking_lot::lock_api::RwLockReadGuard<'_, parking_lot::RawRwLock, PlayerMap> {
        self.players.read()
    }

    pub fn notify_on_player(&self) -> Arc<Notify> {
        self.players_notify.clone()
    }
    pub fn get_player_connection_id(&self, plid: PlayerId) -> Option<ConnectionId> {
        self.idx_player_connection.read().get(&plid).copied()
    }

    pub fn chat(
        &self,
    ) -> parking_lot::lock_api::RwLockReadGuard<'_, parking_lot::RawRwLock, ChatHistory> {
        self.chat.read()
    }

    pub fn notify_on_chat(&self) -> Arc<Notify> {
        self.chat_notify.clone()
    }

    pub(crate) fn handle_data(&self, data: Packet) -> Result<()> {
        match data {
            Packet::MessageOut(data) => {
                self.chat
                    .write()
                    .push_back(chat::Chat::new(data.ucid, data.msg.to_lossy_string()));
                self.chat_notify.notify_waiters();
            }

            Packet::NewConnection(data) => {
                self.connections.write().insert(data.ucid, (&data).into());
                self.chat.write().push_back(chat::Chat::new(
                    data.ucid,
                    format!("New player joined: {}", &data.uname),
                ));
                self.chat_notify.notify_waiters();
            }

            Packet::ConnectionLeave(data) => {
                self.connections.write().remove(&data.ucid);
                if let Some(plid) = self.idx_connection_player.write().remove(&data.ucid) {
                    self.players.write().remove(&plid);
                    self.idx_player_connection.write().remove(&plid);
                }
            }

            Packet::NewPlayer(data) => {
                self.players.write().insert(data.plid, (&data).into());
                self.idx_player_connection
                    .write()
                    .insert(data.plid, data.ucid);
                self.idx_connection_player
                    .write()
                    .insert(data.ucid, data.plid);
            }

            Packet::PlayerLeave(data) => {
                self.players.write().remove(&data.plid);
                if let Some(ucid) = self.idx_player_connection.write().remove(&data.plid) {
                    self.idx_connection_player.write().remove(&ucid);
                }
            }

            Packet::PlayerPits(data) => {
                // Telepits
                self.players.write().remove(&data.plid);
                self.idx_player_connection.write().remove(&data.plid);
            }

            Packet::TakeOverCar(data) => {
                self.idx_player_connection.write().remove(&data.plid);
                self.idx_connection_player.write().remove(&data.olducid);

                self.idx_player_connection
                    .write()
                    .insert(data.plid, data.newucid);
                self.idx_connection_player
                    .write()
                    .insert(data.newucid, data.plid);
            }

            Packet::PitLane(data) => {
                if let Some(player) = self.players.write().get_mut(&data.plid) {
                    player.in_pitlane = data.entered();
                }
            }

            Packet::MultiCarInfo(data) => {
                let mut players = self.players.write();
                for info in data.info.iter() {
                    if let Some(player) = players.get_mut(&info.plid) {
                        player.xyz = info.xyz;
                        player.lap = info.lap;
                        player.position = info.position;
                        player.node = info.node;
                        player.speed = info.speed;
                    }
                }

                self.players_notify.notify_waiters();
            }

            Packet::Lap(data) => {
                if let Some(player) = self.players.write().get_mut(&data.plid) {
                    player.lap = data.lapsdone;
                }
            }

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
            }

            Event::Data(packet) => {
                self.handle_data(packet)?;
            }

            Event::Error(data) => {
                // FIXME
                panic!("{:?}", data);
            }

            Event::Shutdown => panic!("shutdown!"),

            Event::Handshaking => {}

            Event::Disconnected => {
                tracing::debug!("disconnected?!")
            }
        };

        Ok(())
    }
}
