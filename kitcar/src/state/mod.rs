//! Game, connection, player state

use std::{
    collections::{HashMap, HashSet},
    time::Duration,
};

use insim::{
    core::{track::Track, vehicle::Vehicle, wind::Wind},
    identifiers::{ConnectionId, PlayerId},
    insim::{Cnl, Ncn, PlayerFlags, PlayerType, RaceInProgress, RaceLaps, StaFlags},
    Packet, WithRequestId,
};
use tokio::time::{interval, Interval, MissedTickBehavior};

use crate::ui::manager::UIManager;

#[derive(Debug, Default, Clone)]
/// GameInfo
pub struct GameInfo {
    /// Track
    pub track: Option<Track>,

    /// Weather
    pub weather: Option<u8>,

    /// Wind
    pub wind: Option<Wind>,

    /// Race
    pub racing: RaceInProgress,

    /// Qualifying
    pub qualifying_duration: Duration,

    /// Race Duration
    pub race_duration: RaceLaps,

    /// flags
    pub flags: StaFlags,
}

#[derive(Debug, Clone)]
/// PlayerInfo
pub struct PlayerInfo {
    /// PlayerId
    pub plid: PlayerId,

    /// ConnectionId
    pub ucid: ConnectionId,

    /// Vehicle
    pub vehicle: Vehicle,

    /// PlayerType
    pub ptype: PlayerType,

    /// PlayerFlags
    pub flags: PlayerFlags,

    /// In pitlane?
    pub in_pitlane: bool,
}

#[derive(Debug, Clone)]
/// ConnectionInfo
pub struct ConnectionInfo {
    /// ConnectionId
    pub ucid: ConnectionId,

    /// LFS username
    pub uname: String,

    /// Player display name
    pub pname: String,

    /// Admin?
    pub admin: bool,

    /// List of players relating to this connection
    /// Some may be AI players.
    pub players: HashSet<PlayerId>,
}

/// Game state
#[derive(Debug, Default, Clone)]
pub struct State {
    /// GameInfo
    pub game: GameInfo,

    /// ConnectionInfo
    pub connections: HashMap<ConnectionId, ConnectionInfo>,

    /// PlayerInfo
    pub players: HashMap<PlayerId, PlayerInfo>,
}

impl State {
    pub(crate) fn handle_packet(&mut self, packet: &insim::Packet) {
        match packet {
            // Game
            insim::Packet::Sta(sta) => self.sta(sta),
            insim::Packet::Tiny(tiny) => self.tiny(tiny),
            // Connection
            insim::Packet::Ncn(ncn) => self.ncn(ncn),
            insim::Packet::Cnl(cnl) => self.cnl(cnl),
            insim::Packet::Cpr(cpr) => self.cpr(cpr),
            // Player
            insim::Packet::Npl(npl) => self.npl(npl),
            insim::Packet::Pll(pll) => self.pll(pll),
            insim::Packet::Toc(toc) => self.toc(toc),
            insim::Packet::Pfl(pfl) => self.pfl(pfl),
            insim::Packet::Pla(pla) => self.pla(pla),

            // FIXME: process Btn's and pipe through ucid ui to translate and then forward onto the
            // Engine's
            _ => {},
        }
    }

    fn sta(&mut self, sta: &insim::insim::Sta) {
        self.game.racing = sta.raceinprog.clone();
        self.game.qualifying_duration = Duration::from_secs(sta.qualmins as u64 * 60);
        self.game.race_duration = sta.racelaps.clone();

        self.game.track = Some(sta.track.clone());
        self.game.weather = Some(sta.weather);
        self.game.wind = Some(sta.wind);

        self.game.flags = sta.flags.clone();
    }

    fn tiny(&mut self, tiny: &insim::insim::Tiny) {
        if matches!(tiny.subt, insim::insim::TinyType::Clr) {
            self.players.clear();
        }
    }

    fn ncn(&mut self, ncn: &insim::insim::Ncn) {
        let _ = self.connections.insert(
            ncn.ucid.clone(),
            ConnectionInfo {
                ucid: ncn.ucid.clone(),
                admin: ncn.admin,
                uname: ncn.uname.clone(),
                pname: ncn.pname.clone(),
                players: HashSet::new(),
            },
        );
    }

    fn cnl(&mut self, cnl: &insim::insim::Cnl) {
        if let Some(connection) = self.connections.remove(&cnl.ucid) {
            // Remove all players associated with this connection.
            for plid in connection.players {
                let _ = self.players.remove(&plid);
            }
        }
    }

    fn cpr(&mut self, cpr: &insim::insim::Cpr) {
        if let Some(connection) = self.connections.get_mut(&cpr.ucid) {
            connection.pname = cpr.pname.clone();
        }
    }

    fn npl(&mut self, npl: &insim::insim::Npl) {
        let _ = self.players.insert(
            npl.plid.clone(),
            PlayerInfo {
                plid: npl.plid.clone(),
                ucid: npl.ucid.clone(),
                vehicle: npl.cname.clone(),
                ptype: npl.ptype.clone(),
                flags: npl.flags.clone(),
                in_pitlane: false,
            },
        );

        if let Some(connection) = self.connections.get_mut(&npl.ucid) {
            let _ = connection.players.insert(npl.plid.clone());
        }
    }

    fn pll(&mut self, pll: &insim::insim::Pll) {
        if let Some(player) = self.players.remove(&pll.plid) {
            if let Some(connection) = self.connections.get_mut(&player.ucid) {
                let _ = connection.players.remove(&player.plid);
            }
        }
    }

    fn toc(&mut self, toc: &insim::insim::Toc) {
        if let Some(player) = self.players.get_mut(&toc.plid) {
            player.ucid = toc.newucid.clone();
        }

        if let Some(old) = self.connections.get_mut(&toc.olducid) {
            old.players.retain(|&p| p != toc.plid);
        }

        if let Some(new) = self.connections.get_mut(&toc.newucid) {
            let _ = new.players.insert(toc.plid.clone());
        }
    }

    fn pfl(&mut self, pfl: &insim::insim::Pfl) {
        if let Some(player) = self.players.get_mut(&pfl.plid) {
            player.flags = pfl.flags.clone();
        }
    }

    fn pla(&mut self, pla: &insim::insim::Pla) {
        if let Some(player) = self.players.get_mut(&pla.plid) {
            if pla.entered_pitlane() {
                player.in_pitlane = true;
            }

            if pla.exited_pitlane() {
                player.in_pitlane = false;
            }
        }
    }
}

#[derive(Debug)]
pub(crate) struct Ui {
    pub(crate) inner: HashMap<ConnectionId, UIManager>,
    render_interval: Interval,
}

impl Ui {
    pub(crate) fn new(render_interval: Duration) -> Self {
        let mut render_interval = interval(render_interval);
        render_interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

        Self {
            inner: HashMap::new(),
            render_interval,
        }
    }

    pub(crate) fn handle_packet(&mut self, packet: &insim::Packet) {
        match packet {
            // Connection left
            insim::Packet::Ncn(ncn) => self.ncn(ncn),
            insim::Packet::Cnl(cnl) => self.cnl(cnl),

            _ => {},
        }
    }

    fn ncn(&mut self, ncn: &Ncn) {
        let _ = self.inner.insert(ncn.ucid, UIManager::new());
    }

    fn cnl(&mut self, cnl: &Cnl) {
        let _ = self.inner.remove(&cnl.ucid);
    }

    pub(crate) fn render(&mut self) -> Vec<Packet> {
        // FIXME we can do better, but this'll do
        let (to_add, to_remove) = self
            .inner
            .iter_mut()
            .map(|(c, mngr)| mngr.render_all(*c))
            .fold((vec![], vec![]), |mut acc, i| {
                acc.0.extend_from_slice(&i.0);
                acc.1.extend_from_slice(&i.1);
                acc
            });

        let to_remove = to_remove.into_iter().map(|p| {
            let reqi = p.clickid.0;
            p.with_request_id(reqi).into()
        });

        let to_add = to_add.into_iter().map(|p| {
            let reqi = p.clickid.0;
            p.with_request_id(reqi).into()
        });

        to_remove.chain(to_add).collect()
    }

    // Async method that waits for the next tick and then renders
    pub(crate) async fn tick(&mut self) -> Vec<Packet> {
        let _ = self.render_interval.tick().await;
        self.render()
    }
}
