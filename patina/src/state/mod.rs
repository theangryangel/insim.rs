pub(crate) mod chat;
pub(crate) mod player;

use bounded_vec_deque::BoundedVecDeque;
use insim::protocol::identifiers::ConnectionId;
use insim::{client::prelude::*, protocol::identifiers::PlayerId};
use miette::{IntoDiagnostic, Result};
use parking_lot::RwLock;
use std::sync::Arc;
use tokio::sync::{mpsc, Notify};

type ChatHistory = BoundedVecDeque<chat::Chat>;

use indexed_slab::IndexedSlab;
use insim::protocol::position::Point;
use serde::Serialize;

#[derive(IndexedSlab, Default, Debug, Clone, Serialize)]
pub struct Connection {
    #[indexed_slab(ordered, unique)]
    pub connection_id: ConnectionId,

    #[indexed_slab(ordered, unique)]
    pub player_id: Option<PlayerId>,

    /// Connection username
    pub uname: String,

    /// Connection has admin rights
    pub admin: bool,

    /// Connection flags
    pub connection_flags: u8,

    /// player name
    pub pname: String,

    /// player plate
    pub plate: Option<String>,

    pub xyz: Option<Point<i32>>,

    pub in_pitlane: bool,

    pub lap: Option<u16>,

    pub position: Option<u8>,

    pub node: u16,

    pub speed: u16,
}

impl From<&insim::protocol::insim::Ncn> for Connection {
    fn from(data: &insim::protocol::insim::Ncn) -> Self {
        Self {
            uname: data.uname.clone(),
            admin: data.admin > 0,
            connection_flags: data.flags,
            connection_id: data.ucid,
            player_id: None,
            pname: data.pname.to_string(),
            ..Default::default()
        }
    }
}

pub struct State {
    slab: RwLock<IndexedSlabConnection>,

    connections_notify: Arc<Notify>,

    players_notify: Arc<Notify>,

    chat: RwLock<ChatHistory>,
    chat_notify: Arc<Notify>,

    tx: mpsc::UnboundedSender<Event>,
}

#[allow(unused)]
impl State {
    pub fn new(tx: mpsc::UnboundedSender<Event>) -> Self {
        Self {
            slab: RwLock::new(IndexedSlabConnection::default()),
            chat: RwLock::new(BoundedVecDeque::new(256)),
            tx,
            chat_notify: Arc::new(Notify::new()),
            connections_notify: Arc::new(Notify::new()),
            players_notify: Arc::new(Notify::new()),
        }
    }

    pub fn get_connections(&self) -> Vec<Connection> {
        self.slab.write().iter_by_connection_id().cloned().collect()
    }

    pub fn notify_on_connection(&self) -> Arc<Notify> {
        self.connections_notify.clone()
    }

    pub fn get_players(&self) -> Vec<Connection> {
        self.slab
            .write()
            .iter_by_player_id()
            .filter(|c| c.player_id.is_some())
            .cloned()
            .collect()
    }

    pub fn notify_on_player(&self) -> Arc<Notify> {
        self.players_notify.clone()
    }

    pub fn chat(&self) -> ChatHistory {
        (*self.chat.read()).clone()
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
                self.slab.write().insert((&data).into());
                self.chat.write().push_back(chat::Chat::new(
                    data.ucid,
                    format!("New player joined: {}", &data.uname),
                ));
                self.chat_notify.notify_waiters();
                self.connections_notify.notify_waiters();
            }

            Packet::ConnectionLeave(data) => {
                self.slab.write().remove_by_connection_id(&data.ucid);
                self.connections_notify.notify_waiters();
            }

            Packet::NewPlayer(data) => {
                self.slab.write().modify_by_connection_id(&data.ucid, |c| {
                    c.pname = data.pname.to_string();
                    c.plate = Some(data.plate.to_string());
                    c.player_id = Some(data.plid);
                });
                self.players_notify.notify_waiters();
            }

            Packet::PlayerLeave(data) => {
                // FIXME
                self.slab
                    .write()
                    .modify_by_player_id(&Some(data.plid), |c| {
                        c.plate = None;
                        c.player_id = None;
                    });
                self.players_notify.notify_waiters();
            }

            Packet::PlayerPits(data) => {
                // Telepits
                self.slab
                    .write()
                    .modify_by_player_id(&Some(data.plid), |c| {
                        c.plate = None;
                        c.player_id = None;
                    });
                self.players_notify.notify_waiters();
            }

            Packet::TakeOverCar(data) => {
                self.slab
                    .write()
                    .modify_by_player_id(&Some(data.plid), |c| {
                        c.plate = None;
                        c.player_id = None;
                    });

                self.slab
                    .write()
                    .modify_by_connection_id(&data.newucid, |c| {
                        c.player_id = Some(data.plid);
                    });
                self.players_notify.notify_waiters();
            }

            Packet::PitLane(data) => {
                self.slab
                    .write()
                    .modify_by_player_id(&Some(data.plid), |c| c.in_pitlane = data.entered());
            }

            Packet::MultiCarInfo(data) => {
                for info in data.info.iter() {
                    self.slab
                        .write()
                        .modify_by_player_id(&Some(info.plid), |c| {
                            c.xyz = Some(info.xyz);
                            c.lap = Some(info.lap);
                            c.position = Some(info.position);
                            c.node = info.node;
                            c.speed = info.speed;
                        });

                    if (info
                        .info
                        .contains(insim::protocol::insim::CompCarInfo::LAST))
                    {
                        // batch notifications into each set of mci packets
                        self.players_notify.notify_waiters()
                    }
                }
            }

            Packet::Lap(data) => {
                self.slab
                    .write()
                    .modify_by_player_id(&Some(data.plid), |c| {
                        c.lap = Some(data.lapsdone);
                    });
                self.players_notify.notify_waiters();
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
