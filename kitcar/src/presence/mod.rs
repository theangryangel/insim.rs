//! Connection and player presence/tracking

use std::collections::{HashMap, HashSet};

use insim::{
    core::vehicle::Vehicle,
    identifiers::{ConnectionId, PlayerId},
    insim::{PlayerFlags, PlayerType},
};
use tokio::{
    sync::{mpsc, oneshot, watch},
    task::JoinHandle,
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
struct PresenceInner {
    connections: HashMap<ConnectionId, ConnectionInfo>,
    players: HashMap<PlayerId, PlayerInfo>,
    last_known_names: HashMap<String, String>,
    player_count: watch::Sender<usize>,
    connection_count: watch::Sender<usize>,
}

impl PresenceInner {
    /// New presence handler
    fn new() -> Self {
        let player_count = watch::channel(0);
        let connection_count = watch::channel(0);
        Self {
            connections: HashMap::new(),
            players: HashMap::new(),
            last_known_names: HashMap::new(),
            player_count: player_count.0,
            connection_count: connection_count.0,
        }
    }

    /// Fetch all connections
    fn connections(&self) -> impl Iterator<Item = &ConnectionInfo> {
        self.connections.values()
    }

    /// Fetch one connection
    fn connection(&self, ucid: &ConnectionId) -> Option<&ConnectionInfo> {
        self.connections.get(ucid)
    }

    /// Fetch one connection by player
    fn connection_by_player(&self, plid: &PlayerId) -> Option<&ConnectionInfo> {
        let player = self.players.get(plid);
        if let Some(player) = player {
            self.connections.get(&player.ucid)
        } else {
            None
        }
    }

    /// Fetch player count
    fn player_count(&self) -> usize {
        self.players.len()
    }

    /// Fetch all players
    fn players(&self) -> impl Iterator<Item = &PlayerInfo> {
        self.players.values()
    }

    /// Fetch one player
    fn player(&self, plid: &PlayerId) -> Option<&PlayerInfo> {
        self.players.get(plid)
    }

    /// Fetch last known display name by uname
    fn last_known_name(&self, uname: &str) -> Option<&String> {
        self.last_known_names.get(uname)
    }

    /// Handle a game packet
    fn handle_packet(&mut self, packet: &insim::Packet) {
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

        // TODO: be smarter about this
        let _ = self.player_count.send(self.players.len());
        let _ = self.connection_count.send(self.connections.len());
    }

    fn tiny(&mut self, tiny: &insim::insim::Tiny) {
        if matches!(tiny.subt, insim::insim::TinyType::Clr) {
            self.players.clear();
            for conn in self.connections.values_mut() {
                conn.players.clear();
            }
        }
    }

    fn ncn(&mut self, ncn: &insim::insim::Ncn) {
        let _ = self
            .last_known_names
            .insert(ncn.uname.clone(), ncn.pname.clone());

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

            let _ = self
                .last_known_names
                .insert(connection.uname.clone(), cpr.pname.clone());
        }
    }

    fn npl(&mut self, npl: &insim::insim::Npl) {
        let _ = self.players.insert(
            npl.plid,
            PlayerInfo {
                plid: npl.plid,
                ucid: npl.ucid,
                vehicle: npl.cname,
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
        if let Some(player) = self.players.remove(&pll.plid)
            && let Some(connection) = self.connections.get_mut(&player.ucid)
        {
            let _ = connection.players.remove(&player.plid);
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

#[derive(Debug, thiserror::Error)]
/// PresenceError
pub enum PresenceError {
    /// Insim subscription failed
    #[error("Insim subscription failed")]
    SubscriptionFailed,
}

/// Spawn a background instance of Presence and return a handle so that we can query it
pub fn spawn(
    insim: insim::builder::InsimTask,
    capacity: usize,
) -> (Presence, JoinHandle<Result<(), PresenceError>>) {
    let (query_tx, mut query_rx) = mpsc::channel(capacity);
    let mut inner = PresenceInner::new();
    let player_count = inner.player_count.subscribe();
    let connection_count = inner.connection_count.subscribe();

    let handle = tokio::spawn(async move {
        let result: Result<(), PresenceError> = async {
            let mut packet_rx = insim.subscribe();

            loop {
                tokio::select! {
                    Ok(packet) = packet_rx.recv() => {
                        inner.handle_packet(&packet);
                    }
                    Some(query) = query_rx.recv() => {
                        match query {
                            PresenceQuery::Connections { response_tx } => {
                                let _ = response_tx.send(inner.connections().cloned().collect());
                            },
                            PresenceQuery::Connection { ucid, response_tx } => {
                                let _ = response_tx.send(inner.connection(&ucid).cloned());
                            },
                            PresenceQuery::ConnectionByPlayer { plid, response_tx } => {
                                let _ = response_tx.send(inner.connection_by_player(&plid).cloned());
                            },
                            PresenceQuery::Players { response_tx } => {
                                let _ = response_tx.send(inner.players().cloned().collect());
                            },
                            PresenceQuery::Player { plid, response_tx } => {
                                let _ = response_tx.send(inner.player(&plid).cloned());
                            },
                            PresenceQuery::PlayerCount { response_tx } => {
                                let _ = response_tx.send(inner.player_count());
                            }
                            PresenceQuery::LastKnownName { uname, response_tx } => {
                                let _ = response_tx.send(inner.last_known_name(&uname).cloned());
                            }
                            PresenceQuery::LastKnownNames { unames, response_tx } => {
                                let results = unames
                                    .iter()
                                    .filter_map(|uname| {
                                        inner.last_known_name(uname).map(|pname| (uname.clone(), pname.clone()))
                                    })
                                    .collect();
                                let _ = response_tx.send(results);
                            }
                        }
                    }
                }
            }
        }
        .await;
        result
    });

    (
        Presence {
            query_tx,
            player_count,
            connection_count,
        },
        handle,
    )
}

#[derive(Debug)]
enum PresenceQuery {
    Connections {
        response_tx: oneshot::Sender<Vec<ConnectionInfo>>,
    },
    Connection {
        ucid: ConnectionId,
        response_tx: oneshot::Sender<Option<ConnectionInfo>>,
    },
    ConnectionByPlayer {
        plid: PlayerId,
        response_tx: oneshot::Sender<Option<ConnectionInfo>>,
    },
    Players {
        response_tx: oneshot::Sender<Vec<PlayerInfo>>,
    },
    Player {
        plid: PlayerId,
        response_tx: oneshot::Sender<Option<PlayerInfo>>,
    },
    PlayerCount {
        response_tx: oneshot::Sender<usize>,
    },
    LastKnownName {
        uname: String,
        response_tx: oneshot::Sender<Option<String>>,
    },
    LastKnownNames {
        unames: Vec<String>,
        response_tx: oneshot::Sender<HashMap<String, String>>,
    },
}

#[derive(Debug, Clone)]
/// Handler for Presence
pub struct Presence {
    query_tx: mpsc::Sender<PresenceQuery>,
    player_count: watch::Receiver<usize>,
    connection_count: watch::Receiver<usize>,
}

impl Presence {
    /// Watch connection count
    pub async fn wait_for_connection_count(&mut self, f: impl FnMut(&usize) -> bool) -> usize {
        // FIXME
        *self
            .connection_count
            .wait_for(f)
            .await
            .expect("watch connection count wait_for failed")
    }

    /// Watch player count
    pub async fn wait_for_player_count(&mut self, f: impl FnMut(&usize) -> bool) -> usize {
        // FIXME
        *self
            .player_count
            .wait_for(f)
            .await
            .expect("watch player count wait_for failed")
    }

    /// Player count
    pub async fn player_count(&self) -> usize {
        let (tx, rx) = oneshot::channel();
        self.query_tx
            .send(PresenceQuery::PlayerCount { response_tx: tx })
            .await
            .unwrap_or_default();
        rx.await.unwrap_or_default()
    }

    /// get all connections
    pub async fn connections(&self) -> Option<Vec<ConnectionInfo>> {
        let (tx, rx) = oneshot::channel();
        self.query_tx
            .send(PresenceQuery::Connections { response_tx: tx })
            .await
            .ok()?;
        rx.await.ok()
    }

    /// get a connection
    pub async fn connection(&self, ucid: &ConnectionId) -> Option<ConnectionInfo> {
        let (tx, rx) = oneshot::channel();
        self.query_tx
            .send(PresenceQuery::Connection {
                ucid: *ucid,
                response_tx: tx,
            })
            .await
            .ok()?;
        rx.await.ok()?
    }
    /// get a connection by player
    pub async fn connection_by_player(&self, plid: &PlayerId) -> Option<ConnectionInfo> {
        let (tx, rx) = oneshot::channel();
        self.query_tx
            .send(PresenceQuery::ConnectionByPlayer {
                plid: *plid,
                response_tx: tx,
            })
            .await
            .ok()?;
        rx.await.ok()?
    }

    /// get all players
    pub async fn players(&self) -> Option<Vec<PlayerInfo>> {
        let (tx, rx) = oneshot::channel();
        self.query_tx
            .send(PresenceQuery::Players { response_tx: tx })
            .await
            .ok()?;
        rx.await.ok()
    }

    /// get a player
    pub async fn player(&self, plid: &PlayerId) -> Option<PlayerInfo> {
        let (tx, rx) = oneshot::channel();
        self.query_tx
            .send(PresenceQuery::Player {
                plid: *plid,
                response_tx: tx,
            })
            .await
            .ok()?;
        rx.await.ok()?
    }

    /// get last known display name by uname (persists after disconnect)
    pub async fn last_known_name(&self, uname: &str) -> Option<String> {
        let (tx, rx) = oneshot::channel();
        self.query_tx
            .send(PresenceQuery::LastKnownName {
                uname: uname.to_string(),
                response_tx: tx,
            })
            .await
            .ok()?;
        rx.await.ok()?
    }

    /// batch fetch last known display names by unames (persists after disconnect)
    pub async fn last_known_names<I, S>(&self, unames: I) -> Option<HashMap<String, String>>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let (tx, rx) = oneshot::channel();
        self.query_tx
            .send(PresenceQuery::LastKnownNames {
                unames: unames.into_iter().map(Into::into).collect(),
                response_tx: tx,
            })
            .await
            .ok()?;
        rx.await.ok()
    }
}
