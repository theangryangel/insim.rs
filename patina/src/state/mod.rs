pub(crate) mod chat;

use bounded_vec_deque::BoundedVecDeque;
use insim::client::prelude::*;
use insim::core::{identifiers::ConnectionId, identifiers::PlayerId, point::Point};
use insim::protocol::insim::Wind;
use insim::track::TrackInfo;
use miette::Result;
use std::sync::Arc;
use tokio::sync::Notify;

type ChatHistory = BoundedVecDeque<chat::Chat>;

use md5::{Digest, Md5};
use multi_index::MultiIndex;
use serde::Serialize;

fn string_to_hex_colour(input: &str) -> String {
    let mut hasher = Md5::new();
    hasher.update(input);
    let result = hasher.finalize();
    format!("#{:#06.6x}", result)
}

#[derive(MultiIndex, Default, Debug, Clone, Serialize)]
pub struct Connection {
    #[multi_index(how = "ordered", unique)]
    pub connection_id: ConnectionId,

    #[multi_index(how = "ordered", unique, ignore_none)]
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

    pub colour: String,
}

impl From<&insim::protocol::insim::Ncn> for Connection {
    fn from(data: &insim::protocol::insim::Ncn) -> Self {
        Self {
            uname: data.uname.clone(),
            admin: data.admin,
            connection_flags: data.flags,
            connection_id: data.ucid,
            player_id: None,
            pname: data.pname.to_string(),
            colour: string_to_hex_colour(&data.uname),
            ..Default::default()
        }
    }
}

#[derive(Default, Clone, Serialize)]
pub struct Game {
    pub track: Option<TrackInfo>,
    pub weather: u8,
    pub wind: Wind,

    pub racing: bool,
    pub lap_count: u8,
    pub qualifying_duration: u8,
}

#[derive(Clone)]
pub struct Notifiers {
    pub connections: Arc<Notify>,
    pub players: Arc<Notify>,
    pub chat: Arc<Notify>,
}

impl Notifiers {
    pub fn new() -> Self {
        Self {
            connections: Arc::new(Notify::new()),
            players: Arc::new(Notify::new()),
            chat: Arc::new(Notify::new()),
        }
    }
}

pub struct State {
    pub slab: MultiIndexConnection,

    pub game: Game,
    pub chat: ChatHistory,

    pub notifiers: Notifiers,
}

#[allow(unused)]
impl State {
    pub fn new() -> Self {
        Self {
            slab: MultiIndexConnection::default(),
            chat: BoundedVecDeque::new(256),
            game: Game::default(),
            notifiers: Notifiers::new(),
        }
    }

    pub fn get_connections(&self) -> Vec<Connection> {
        self.slab.iter_by_connection_id().cloned().collect()
    }

    pub fn get_players(&self, flipped: bool) -> Vec<Connection> {
        self.slab
            .iter_by_player_id()
            .filter(|c| c.player_id.is_some())
            .map(|c| {
                let c = c.clone();
                if flipped {
                    if let Some(mut xyz) = c.xyz {
                        xyz = xyz.flipped();
                    }
                }
                c
            })
            .collect()
    }

    pub fn chat(&self) -> ChatHistory {
        self.chat.clone()
    }

    pub fn game(&self) -> Game {
        self.game.clone()
    }

    pub(crate) fn handle_packet(&mut self, data: &Packet) -> Result<()> {
        match data {
            Packet::MessageOut(data) => {
                self.chat
                    .push_front(chat::Chat::new(data.ucid, data.msg.to_owned()));
                self.notifiers.chat.notify_waiters();
            }

            Packet::NewConnection(data) => {
                let connection: Connection = (data).into();

                let mut msg =
                    chat::Chat::new(data.ucid, format!("New player joined: {}", &data.uname));

                msg.uname = connection.uname.clone();
                msg.pname = connection.pname.clone();
                msg.colour = connection.colour.clone();

                self.slab.insert(connection);

                self.chat.push_front(msg);
                self.notifiers.chat.notify_waiters();
                self.notifiers.connections.notify_waiters();
            }

            Packet::ConnectionLeave(data) => {
                self.slab.remove_by_connection_id(&data.ucid);
                self.notifiers.connections.notify_waiters();
            }

            Packet::NewPlayer(data) => {
                self.slab.modify_by_connection_id(&data.ucid, |c| {
                    c.pname = data.pname.to_string();
                    c.plate = Some(data.plate.to_string());
                    c.player_id = Some(data.plid);
                });
                self.notifiers.players.notify_waiters();
            }

            Packet::PlayerLeave(data) => {
                // FIXME
                self.slab.modify_by_player_id(&Some(data.plid), |c| {
                    c.plate = None;
                    c.player_id = None;
                });
                self.notifiers.players.notify_waiters();
            }

            Packet::PlayerPits(data) => {
                // Telepits
                self.slab.modify_by_player_id(&Some(data.plid), |c| {
                    c.plate = None;
                    c.player_id = None;
                });
                self.notifiers.players.notify_waiters();
            }

            Packet::TakeOverCar(data) => {
                self.slab.modify_by_player_id(&Some(data.plid), |c| {
                    c.plate = None;
                    c.player_id = None;
                });

                self.slab.modify_by_connection_id(&data.newucid, |c| {
                    c.player_id = Some(data.plid);
                });
                self.notifiers.players.notify_waiters();
            }

            Packet::PitLane(data) => {
                self.slab
                    .modify_by_player_id(&Some(data.plid), |c| c.in_pitlane = data.entered());
            }

            Packet::MultiCarInfo(data) => {
                for info in data.info.iter() {
                    self.slab.modify_by_player_id(&Some(info.plid), |c| {
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
                        self.notifiers.players.notify_waiters()
                    }
                }
            }

            Packet::Lap(data) => {
                self.slab.modify_by_player_id(&Some(data.plid), |c| {
                    c.lap = Some(data.lapsdone);
                });
                self.notifiers.players.notify_waiters();
            }

            Packet::State(data) => {
                let track = &data.track;

                self.game.track = track.track_info();
                self.game.weather = data.weather;
                self.game.wind = data.wind;
                self.game.racing = data.raceinprog > 0;
                self.game.lap_count = data.racelaps;
                self.game.qualifying_duration = data.qualmins;
            }

            _ => {}
        }

        Ok(())
    }

    pub(crate) fn handle_insim_event(&mut self, event: &Event) -> Result<()> {
        match event {
            Event::Connected => {}

            Event::Data(packet) => {
                self.handle_packet(packet)?;
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
