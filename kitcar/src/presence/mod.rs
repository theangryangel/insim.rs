//! Connection and player presence/tracking

use std::collections::{HashMap, HashSet};

use insim::{
    core::vehicle::Vehicle,
    identifiers::{ConnectionId, PlayerId},
    insim::{PlayerFlags, PlayerType},
};

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

    /// Playername
    pub pname: String,
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

/// Presence state
#[derive(Debug, Default, Clone)]
pub struct Presence {
    connections: HashMap<ConnectionId, ConnectionInfo>,
    players: HashMap<PlayerId, PlayerInfo>,
}

impl Presence {
    /// New presence handler
    pub fn new() -> Self {
        Self {
            connections: HashMap::new(),
            players: HashMap::new(),
        }
    }

    /// Fetch all connections
    pub fn connections(&self) -> impl Iterator<Item = &ConnectionInfo> {
        self.connections.values()
    }

    /// Fetch one connection
    pub fn connection(&self, ucid: &ConnectionId) -> Option<&ConnectionInfo> {
        self.connections.get(ucid)
    }

    /// Fetch player count
    pub fn player_count(&self) -> usize {
        self.players.len()
    }

    /// Fetch all players
    pub fn players(&self) -> impl Iterator<Item = &PlayerInfo> {
        self.players.values()
    }

    /// Fetch one player
    pub fn player(&self, plid: &PlayerId) -> Option<&PlayerInfo> {
        self.players.get(plid)
    }

    /// Handle a game packet
    pub fn handle_packet(&mut self, packet: &insim::Packet) {
        match packet {
            // Game
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

    fn tiny(&mut self, tiny: &insim::insim::Tiny) {
        if matches!(tiny.subt, insim::insim::TinyType::Clr) {
            self.players.clear();
        }
    }

    fn ncn(&mut self, ncn: &insim::insim::Ncn) {
        let _ = self.connections.insert(
            ncn.ucid,
            ConnectionInfo {
                ucid: ncn.ucid,
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
            npl.plid,
            PlayerInfo {
                plid: npl.plid,
                ucid: npl.ucid,
                vehicle: npl.cname.clone(),
                ptype: npl.ptype,
                flags: npl.flags,
                in_pitlane: false,
                pname: npl.pname.clone(),
            },
        );

        if let Some(connection) = self.connections.get_mut(&npl.ucid) {
            let _ = connection.players.insert(npl.plid);
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
            player.ucid = toc.newucid;
        }

        if let Some(old) = self.connections.get_mut(&toc.olducid) {
            old.players.retain(|&p| p != toc.plid);
        }

        if let Some(new) = self.connections.get_mut(&toc.newucid) {
            let _ = new.players.insert(toc.plid);
        }
    }

    fn pfl(&mut self, pfl: &insim::insim::Pfl) {
        if let Some(player) = self.players.get_mut(&pfl.plid) {
            player.flags = pfl.flags;
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
