use crate::{core::wind::Wind, game_data::track, packets::Packet};
pub mod connection;

use connection::{Connection, MultiIndexConnection};

#[derive(Clone)]
pub struct GameState {
    pub slab: MultiIndexConnection,

    pub track: Option<track::TrackInfo>,
    pub weather: Option<u8>,
    pub wind: Option<Wind>,

    pub racing: bool,
}

impl Default for GameState {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(unused)]
impl GameState {
    pub fn new() -> Self {
        Self {
            slab: MultiIndexConnection::default(),
            track: None,
            weather: None,
            wind: None,
            racing: false,
        }
    }

    pub fn get_connections(&self) -> Vec<Connection> {
        self.slab.iter_by_connection_id().cloned().collect()
    }

    pub fn get_players(&self) -> Vec<Connection> {
        self.slab
            .iter_by_player_id()
            .filter(|c| c.player_id.is_some())
            .cloned()
            .collect()
    }

    pub fn handle(&mut self, data: &Packet) {
        match data {
            Packet::NewConnection(data) => {
                let connection: Connection = (data).into();
                self.slab.insert(connection);
            }

            Packet::ConnectionLeave(data) => {
                self.slab.remove_by_connection_id(&data.ucid);
            }

            Packet::NewPlayer(data) => {
                self.slab.modify_by_connection_id(&data.ucid, |c| {
                    c.pname = data.pname.to_string();
                    c.plate = Some(data.plate.to_string());
                    c.player_id = Some(data.plid);
                });
            }

            Packet::PlayerLeave(data) => {
                // FIXME
                self.slab.modify_by_player_id(&Some(data.plid), |c| {
                    c.plate = None;
                    c.player_id = None;
                });
            }

            Packet::PlayerPits(data) => {
                // Telepits
                self.slab.modify_by_player_id(&Some(data.plid), |c| {
                    c.plate = None;
                    c.player_id = None;
                });
            }

            Packet::TakeOverCar(data) => {
                self.slab.modify_by_player_id(&Some(data.plid), |c| {
                    c.plate = None;
                    c.player_id = None;
                });

                self.slab.modify_by_connection_id(&data.newucid, |c| {
                    c.player_id = Some(data.plid);
                });
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
                }
            }

            Packet::Lap(data) => {
                self.slab.modify_by_player_id(&Some(data.plid), |c| {
                    c.lap = Some(data.lapsdone);
                });
            }

            Packet::State(data) => {
                let track = &data.track;

                self.track = track::lookup(track).cloned();
                self.weather = Some(data.weather);
                self.wind = Some(data.wind);
                self.racing = data.raceinprog > 0;
            }

            _ => {}
        }
    }
}
