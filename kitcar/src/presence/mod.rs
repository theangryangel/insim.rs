//! Connection and player presence/tracking

use std::collections::{HashMap, HashSet};

use insim::{
    core::vehicle::Vehicle,
    identifiers::{ConnectionId, PlayerId},
    insim::{PlayerFlags, PlayerType},
};
use tokio::{
    sync::{broadcast, mpsc, oneshot, watch},
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

/// Lifecycle events emitted by `Presence` after processing each packet.
#[derive(Debug, Clone)]
pub enum PresenceEvent {
    /// A new connection joined.
    Connected(ConnectionInfo),
    /// A connection left.
    Disconnected(ConnectionInfo),
    /// A connection changed their display name.
    Renamed {
        /// Connection ID
        ucid: ConnectionId,
        /// LFS username
        uname: String,
        /// New display name
        new_pname: String,
    },
    /// A player joined the track.
    PlayerJoined(PlayerInfo),
    /// A player left the track.
    PlayerLeft(PlayerInfo),
}

/// Presence state
#[derive(Debug)]
struct PresenceInner {
    connections: HashMap<ConnectionId, ConnectionInfo>,
    players: HashMap<PlayerId, PlayerInfo>,
    last_known_names: HashMap<String, String>,
    player_count: watch::Sender<usize>,
    connection_count: watch::Sender<usize>,
    event_tx: broadcast::Sender<PresenceEvent>,
}

impl PresenceInner {
    /// New presence handler
    fn new() -> Self {
        let player_count = watch::channel(0);
        let connection_count = watch::channel(0);
        let (event_tx, _) = broadcast::channel(64);
        Self {
            connections: HashMap::new(),
            players: HashMap::new(),
            last_known_names: HashMap::new(),
            player_count: player_count.0,
            connection_count: connection_count.0,
            event_tx,
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
        let old_player_count = self.players.len();
        let old_connection_count = self.connections.len();

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

        let new_player_count = self.players.len();
        if new_player_count != old_player_count {
            let _ = self.player_count.send(new_player_count);
        }

        let new_connection_count = self.connections.len();
        if new_connection_count != old_connection_count {
            let _ = self.connection_count.send(new_connection_count);
        }
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

        let conn = ConnectionInfo {
            ucid: ncn.ucid,
            admin: ncn.admin,
            uname: ncn.uname.clone(),
            pname: ncn.pname.clone(),
            players: HashSet::new(),
        };

        let _ = self.event_tx.send(PresenceEvent::Connected(conn.clone()));
        let _ = self.connections.insert(ncn.ucid, conn);
    }

    fn cnl(&mut self, cnl: &insim::insim::Cnl) {
        if let Some(connection) = self.connections.remove(&cnl.ucid) {
            // Remove all players associated with this connection.
            for plid in &connection.players {
                if let Some(player) = self.players.remove(plid) {
                    let _ = self.event_tx.send(PresenceEvent::PlayerLeft(player));
                }
            }
            let _ = self.event_tx.send(PresenceEvent::Disconnected(connection));
        }
    }

    fn cpr(&mut self, cpr: &insim::insim::Cpr) {
        if let Some(connection) = self.connections.get_mut(&cpr.ucid) {
            connection.pname = cpr.pname.clone();

            let _ = self
                .last_known_names
                .insert(connection.uname.clone(), cpr.pname.clone());

            let _ = self.event_tx.send(PresenceEvent::Renamed {
                ucid: cpr.ucid,
                uname: connection.uname.clone(),
                new_pname: cpr.pname.clone(),
            });
        }
    }

    fn npl(&mut self, npl: &insim::insim::Npl) {
        if npl.nump == 0 {
            // A join request is seen as an IS_NPL packet with ZERO in the NumP field
            // An immediate response (e.g. within 1 second) is required using an IS_JRR packet
            // If you allow the join and it is successful you will then get a normal IS_NPL with NumP set.
            return;
        }

        let player = PlayerInfo {
            plid: npl.plid,
            ucid: npl.ucid,
            vehicle: npl.cname,
            ptype: npl.ptype,
            flags: npl.flags,
            in_pitlane: false,
            pname: npl.pname.clone(),
        };

        let _ = self
            .event_tx
            .send(PresenceEvent::PlayerJoined(player.clone()));
        let _ = self.players.insert(npl.plid, player);

        if let Some(connection) = self.connections.get_mut(&npl.ucid) {
            let _ = connection.players.insert(npl.plid);
        }
    }

    fn pll(&mut self, pll: &insim::insim::Pll) {
        if let Some(player) = self.players.remove(&pll.plid) {
            let _ = self
                .event_tx
                .send(PresenceEvent::PlayerLeft(player.clone()));
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

#[derive(Debug, thiserror::Error)]
/// PresenceError
pub enum PresenceError {
    /// Lost Insim packet stream
    #[error("Lost Insim packet stream")]
    InsimHandleLost,

    /// Lost presence watch channel
    #[error("Lost presence watch channel")]
    WatchChannelClosed,

    /// Lost presence query channel
    #[error("Lost presence query channel")]
    QueryChannelClosed,

    /// Lost presence response channel
    #[error("Lost presence response channel")]
    ResponseChannelClosed,
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
    let event_tx = inner.event_tx.clone();

    let handle = tokio::spawn(async move {
        let result: Result<(), PresenceError> = async {
            let mut packet_rx = insim.subscribe();

            loop {
                tokio::select! {
                    packet = packet_rx.recv() => {
                        match packet {
                            Ok(packet) => inner.handle_packet(&packet),
                            Err(_) => return Err(PresenceError::InsimHandleLost),
                        }
                    }
                    query = query_rx.recv() => {
                        match query {
                            Some(PresenceQuery::Connections { response_tx }) => {
                                let _ = response_tx.send(inner.connections().cloned().collect());
                            },
                            Some(PresenceQuery::Connection { ucid, response_tx }) => {
                                let _ = response_tx.send(inner.connection(&ucid).cloned());
                            },
                            Some(PresenceQuery::ConnectionByPlayer { plid, response_tx }) => {
                                let _ = response_tx.send(inner.connection_by_player(&plid).cloned());
                            },
                            Some(PresenceQuery::Players { response_tx }) => {
                                let _ = response_tx.send(inner.players().cloned().collect());
                            },
                            Some(PresenceQuery::Player { plid, response_tx }) => {
                                let _ = response_tx.send(inner.player(&plid).cloned());
                            },
                            Some(PresenceQuery::PlayerCount { response_tx }) => {
                                let _ = response_tx.send(inner.player_count());
                            }
                            Some(PresenceQuery::LastKnownName { uname, response_tx }) => {
                                let _ = response_tx.send(inner.last_known_name(&uname).cloned());
                            }
                            Some(PresenceQuery::LastKnownNames { unames, response_tx }) => {
                                let results = unames
                                    .iter()
                                    .filter_map(|uname| {
                                        inner
                                            .last_known_name(uname)
                                            .map(|pname| (uname.clone(), pname.clone()))
                                    })
                                    .collect();
                                let _ = response_tx.send(results);
                            }
                            None => break,
                        }
                    }
                }
            }

            Ok(())
        }
        .await;
        result
    });

    (
        Presence {
            query_tx,
            player_count,
            connection_count,
            event_tx,
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
    event_tx: broadcast::Sender<PresenceEvent>,
}

impl Presence {
    /// Subscribe to presence lifecycle events.
    pub fn subscribe_events(&self) -> broadcast::Receiver<PresenceEvent> {
        self.event_tx.subscribe()
    }

    /// Watch connection count
    pub async fn wait_for_connection_count(
        &mut self,
        f: impl FnMut(&usize) -> bool,
    ) -> Result<usize, PresenceError> {
        self.connection_count
            .wait_for(f)
            .await
            .map(|val| *val)
            .map_err(|_| PresenceError::WatchChannelClosed)
    }

    /// Watch player count
    pub async fn wait_for_player_count(
        &mut self,
        f: impl FnMut(&usize) -> bool,
    ) -> Result<usize, PresenceError> {
        self.player_count
            .wait_for(f)
            .await
            .map(|val| *val)
            .map_err(|_| PresenceError::WatchChannelClosed)
    }

    /// Player count
    pub async fn player_count(&self) -> Result<usize, PresenceError> {
        let (tx, rx) = oneshot::channel();
        self.query_tx
            .send(PresenceQuery::PlayerCount { response_tx: tx })
            .await
            .map_err(|_| PresenceError::QueryChannelClosed)?;
        rx.await.map_err(|_| PresenceError::ResponseChannelClosed)
    }

    /// get all connections
    pub async fn connections(&self) -> Result<Vec<ConnectionInfo>, PresenceError> {
        let (tx, rx) = oneshot::channel();
        self.query_tx
            .send(PresenceQuery::Connections { response_tx: tx })
            .await
            .map_err(|_| PresenceError::QueryChannelClosed)?;
        rx.await.map_err(|_| PresenceError::ResponseChannelClosed)
    }

    /// get a connection
    pub async fn connection(
        &self,
        ucid: &ConnectionId,
    ) -> Result<Option<ConnectionInfo>, PresenceError> {
        let (tx, rx) = oneshot::channel();
        self.query_tx
            .send(PresenceQuery::Connection {
                ucid: *ucid,
                response_tx: tx,
            })
            .await
            .map_err(|_| PresenceError::QueryChannelClosed)?;
        rx.await.map_err(|_| PresenceError::ResponseChannelClosed)
    }
    /// get a connection by player
    pub async fn connection_by_player(
        &self,
        plid: &PlayerId,
    ) -> Result<Option<ConnectionInfo>, PresenceError> {
        let (tx, rx) = oneshot::channel();
        self.query_tx
            .send(PresenceQuery::ConnectionByPlayer {
                plid: *plid,
                response_tx: tx,
            })
            .await
            .map_err(|_| PresenceError::QueryChannelClosed)?;
        rx.await.map_err(|_| PresenceError::ResponseChannelClosed)
    }

    /// get all players
    pub async fn players(&self) -> Result<Vec<PlayerInfo>, PresenceError> {
        let (tx, rx) = oneshot::channel();
        self.query_tx
            .send(PresenceQuery::Players { response_tx: tx })
            .await
            .map_err(|_| PresenceError::QueryChannelClosed)?;
        rx.await.map_err(|_| PresenceError::ResponseChannelClosed)
    }

    /// get a player
    pub async fn player(&self, plid: &PlayerId) -> Result<Option<PlayerInfo>, PresenceError> {
        let (tx, rx) = oneshot::channel();
        self.query_tx
            .send(PresenceQuery::Player {
                plid: *plid,
                response_tx: tx,
            })
            .await
            .map_err(|_| PresenceError::QueryChannelClosed)?;
        rx.await.map_err(|_| PresenceError::ResponseChannelClosed)
    }

    /// get last known display name by uname (persists after disconnect)
    pub async fn last_known_name(&self, uname: &str) -> Result<Option<String>, PresenceError> {
        let (tx, rx) = oneshot::channel();
        self.query_tx
            .send(PresenceQuery::LastKnownName {
                uname: uname.to_string(),
                response_tx: tx,
            })
            .await
            .map_err(|_| PresenceError::QueryChannelClosed)?;
        rx.await.map_err(|_| PresenceError::ResponseChannelClosed)
    }

    /// batch fetch last known display names by unames (persists after disconnect)
    pub async fn last_known_names<I, S>(
        &self,
        unames: I,
    ) -> Result<HashMap<String, String>, PresenceError>
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
            .map_err(|_| PresenceError::QueryChannelClosed)?;
        rx.await.map_err(|_| PresenceError::ResponseChannelClosed)
    }
}
