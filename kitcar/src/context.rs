//! Context
use std::{
    collections::{HashMap, HashSet, VecDeque},
    fmt::Debug,
    time::Duration,
};

use insim::{
    core::{track::Track, vehicle::Vehicle, wind::Wind},
    identifiers::{ConnectionId, PlayerId},
    insim::{PlayerFlags, PlayerType, RaceInProgress, RaceLaps, StaFlags},
};

#[derive(Debug, Default)]
/// GameInfo
pub struct Game<S> {
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

    /// Custom State
    pub state: S,
}

#[derive(Debug)]
/// PlayerInfo
pub struct PlayerInfo<S> {
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

    /// Custom State
    pub state: S,
}

#[derive(Debug)]
/// ConnectionInfo
pub struct ConnectionInfo<S> {
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

    /// Custom State
    pub state: S,
}

/// A container for the user-supplied state.
#[derive(Debug)]
pub struct Context<S, P, C, G>
where
    S: Default + Debug,
    P: Default + Debug,
    C: Default + Debug,
    G: Default + Debug,
{
    pub(crate) stop: bool,
    pub(crate) outgoing_packets: VecDeque<insim::Packet>,

    /// Custom global state - useful for things like database connections, etc.
    pub state: S,

    /// GameInfo
    pub game: Game<G>,

    /// ConnectionInfo
    pub connections: HashMap<ConnectionId, ConnectionInfo<C>>,

    /// PlayerInfo
    pub players: HashMap<PlayerId, PlayerInfo<P>>,
}

impl<S, P, C, G> Context<S, P, C, G>
where
    S: Default + Debug,
    P: Default + Debug,
    C: Default + Debug,
    G: Default + Debug,
{
    /// A convenience method to shutdown.
    pub fn shutdown(&mut self) {
        self.stop = true;
    }

    /// A convenience method to queue a packet for later sending.
    pub fn queue_packet<I: Into<insim::Packet>>(&mut self, packet: I) {
        self.outgoing_packets.push_back(packet.into());
    }

    pub(crate) fn packet(&mut self, packet: &insim::Packet) {
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
                state: C::default(),
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
                state: P::default(),
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
