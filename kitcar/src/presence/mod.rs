//! Connection and player presence/tracking

use std::{
    collections::{HashMap, HashSet},
    net::Ipv4Addr,
};

use insim::{
    core::vehicle::Vehicle,
    identifiers::{ConnectionId, PlayerId},
    insim::{PlayerFlags, PlayerType},
};
use tokio::{
    sync::{broadcast, mpsc, oneshot},
    task::JoinHandle,
    time,
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

    /// LFS.net user ID. Only populated when the host has an admin password set
    /// and an IS_NCI packet has been received for this connection.
    pub userid: Option<u32>,

    /// Originating IP address. Only populated on hosts with an admin password.
    pub ipaddress: Option<Ipv4Addr>,

    /// Most recently selected vehicle in the garage (from IS_SLC).
    pub selected_vehicle: Option<Vehicle>,
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
    /// A player's controlling connection changed (driver swap).
    TakingOver {
        /// The player before the swap (with old ucid).
        before: PlayerInfo,
        /// The player after the swap (with new ucid).
        after: PlayerInfo,
    },
    /// Extra connection details received (IS_NCI - host-only, requires admin password).
    /// Enriches the connection with LFS.net user ID and IP address.
    ConnectionDetails(ConnectionInfo),
    /// A connection selected a vehicle in the garage (IS_SLC).
    VehicleSelected {
        /// Connection that selected the vehicle.
        ucid: ConnectionId,
        /// The selected vehicle.
        vehicle: Vehicle,
    },
    /// A player tele-pitted (Shift+P). The player is still in the race but repositioned.
    PlayerTeleportedToPits(PlayerInfo),
}

/// Presence state
#[derive(Debug)]
struct PresenceInner {
    connections: HashMap<ConnectionId, ConnectionInfo>,
    players: HashMap<PlayerId, PlayerInfo>,
    last_known_names: HashMap<String, String>,
    event_tx: broadcast::Sender<PresenceEvent>,
}

impl PresenceInner {
    /// New presence handler
    fn new() -> Self {
        let (event_tx, _) = broadcast::channel(64);
        Self {
            connections: HashMap::new(),
            players: HashMap::new(),
            last_known_names: HashMap::new(),
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
        match packet {
            // Game
            insim::Packet::Tiny(tiny) => self.tiny(tiny),
            // Connection
            insim::Packet::Ncn(ncn) => self.ncn(ncn),
            insim::Packet::Nci(nci) => self.nci(nci),
            insim::Packet::Cnl(cnl) => self.cnl(cnl),
            insim::Packet::Cpr(cpr) => self.cpr(cpr),
            insim::Packet::Slc(slc) => self.slc(slc),
            // Player
            insim::Packet::Npl(npl) => self.npl(npl),
            insim::Packet::Pll(pll) => self.pll(pll),
            insim::Packet::Plp(plp) => self.plp(plp),
            insim::Packet::Toc(toc) => self.toc(toc),
            insim::Packet::Pfl(pfl) => self.pfl(pfl),
            insim::Packet::Pla(pla) => self.pla(pla),

            _ => {},
        }
    }

    fn connection_count(&self) -> usize {
        self.connections.len()
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
            userid: None,
            ipaddress: None,
            selected_vehicle: None,
        };

        let _ = self.event_tx.send(PresenceEvent::Connected(conn.clone()));
        let _ = self.connections.insert(ncn.ucid, conn);
    }

    fn nci(&mut self, nci: &insim::insim::Nci) {
        if let Some(connection) = self.connections.get_mut(&nci.ucid) {
            connection.userid = Some(nci.userid);
            connection.ipaddress = Some(nci.ipaddress);
            let _ = self
                .event_tx
                .send(PresenceEvent::ConnectionDetails(connection.clone()));
        }
    }

    fn slc(&mut self, slc: &insim::insim::Slc) {
        if let Some(connection) = self.connections.get_mut(&slc.ucid) {
            connection.selected_vehicle = Some(slc.cname);
            let _ = self.event_tx.send(PresenceEvent::VehicleSelected {
                ucid: slc.ucid,
                vehicle: slc.cname,
            });
        }
    }

    fn plp(&mut self, plp: &insim::insim::Plp) {
        if let Some(player) = self.players.get(&plp.plid) {
            let _ = self
                .event_tx
                .send(PresenceEvent::PlayerTeleportedToPits(player.clone()));
        }
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
            let before = player.clone();
            player.ucid = toc.newucid;
            let after = player.clone();
            let _ = self
                .event_tx
                .send(PresenceEvent::TakingOver { before, after });
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
    /// Insim error
    #[error("Insim: {0}")]
    Insim(#[from] insim::Error),

    /// Lost Insim packet stream
    #[error("Lost Insim packet stream")]
    InsimHandleLost,

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
    let (tx, mut rx) = mpsc::channel(capacity);
    let mut inner = PresenceInner::new();
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
                    msg = rx.recv() => {
                        match msg {
                            Some(PresenceMessage::Connections { response_tx }) => {
                                let _ = response_tx.send(inner.connections().cloned().collect());
                            },
                            Some(PresenceMessage::Connection { ucid, response_tx }) => {
                                let _ = response_tx.send(inner.connection(&ucid).cloned());
                            },
                            Some(PresenceMessage::ConnectionByPlayer { plid, response_tx }) => {
                                let _ = response_tx.send(inner.connection_by_player(&plid).cloned());
                            },
                            Some(PresenceMessage::Players { response_tx }) => {
                                let _ = response_tx.send(inner.players().cloned().collect());
                            },
                            Some(PresenceMessage::Player { plid, response_tx }) => {
                                let _ = response_tx.send(inner.player(&plid).cloned());
                            },
                            Some(PresenceMessage::PlayerCount { response_tx }) => {
                                let _ = response_tx.send(inner.player_count());
                            },
                            Some(PresenceMessage::ConnectionCount { response_tx }) => {
                                let _ = response_tx.send(inner.connection_count());
                            },
                            Some(PresenceMessage::LastKnownName { uname, response_tx }) => {
                                let _ = response_tx.send(inner.last_known_name(&uname).cloned());
                            }
                            Some(PresenceMessage::LastKnownNames { unames, response_tx }) => {
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
                            Some(PresenceMessage::Kick { ucid, response_tx }) => {
                                let res = match inner.connection(&ucid) {
                                    Some(conn) => insim
                                        .send_command(format!("/kick {}", conn.uname))
                                        .await
                                        .map_err(PresenceError::from),
                                    None => Ok(()),
                                };
                                let _ = response_tx.send(res);
                            }
                            Some(PresenceMessage::Ban { ucid, ban_days, response_tx }) => {
                                let res = match inner.connection(&ucid) {
                                    Some(conn) => insim
                                        .send_command(format!("/ban {} {ban_days}", conn.uname))
                                        .await
                                        .map_err(PresenceError::from),
                                    None => Ok(()),
                                };
                                let _ = response_tx.send(res);
                            }
                            Some(PresenceMessage::Unban { uname, response_tx }) => {
                                let res = insim
                                    .send_command(format!("/unban {uname}"))
                                    .await
                                    .map_err(PresenceError::from);
                                let _ = response_tx.send(res);
                            }
                            Some(PresenceMessage::Spec { ucid, response_tx }) => {
                                let res = match inner.connection(&ucid) {
                                    Some(conn) => insim
                                        .send_command(format!("/spec {}", conn.uname))
                                        .await
                                        .map_err(PresenceError::from),
                                    None => Ok(()),
                                };
                                let _ = response_tx.send(res);
                            }
                            Some(PresenceMessage::Pitlane { ucid, response_tx }) => {
                                let res = match inner.connection(&ucid) {
                                    Some(conn) => insim
                                        .send_command(format!("/pitlane {}", conn.uname))
                                        .await
                                        .map_err(PresenceError::from),
                                    None => Ok(()),
                                };
                                let _ = response_tx.send(res);
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

    (Presence { tx, event_tx }, handle)
}

#[derive(Debug)]
enum PresenceMessage {
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
    ConnectionCount {
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
    Kick {
        ucid: ConnectionId,
        response_tx: oneshot::Sender<Result<(), PresenceError>>,
    },
    Ban {
        ucid: ConnectionId,
        ban_days: u32,
        response_tx: oneshot::Sender<Result<(), PresenceError>>,
    },
    Unban {
        uname: String,
        response_tx: oneshot::Sender<Result<(), PresenceError>>,
    },
    Spec {
        ucid: ConnectionId,
        response_tx: oneshot::Sender<Result<(), PresenceError>>,
    },
    Pitlane {
        ucid: ConnectionId,
        response_tx: oneshot::Sender<Result<(), PresenceError>>,
    },
}

#[derive(Debug, Clone)]
/// Handler for Presence
pub struct Presence {
    tx: mpsc::Sender<PresenceMessage>,
    event_tx: broadcast::Sender<PresenceEvent>,
}

impl Presence {
    /// Subscribe to presence lifecycle events.
    pub fn subscribe_events(&self) -> broadcast::Receiver<PresenceEvent> {
        self.event_tx.subscribe()
    }

    /// Poll until the connection count satisfies the predicate.
    ///
    /// `poll_interval` controls how frequently the predicate is evaluated.
    pub async fn wait_for_connection_count(
        &self,
        f: impl Fn(usize) -> bool,
        poll_interval: std::time::Duration,
    ) -> Result<usize, PresenceError> {
        let mut interval = time::interval(poll_interval);
        loop {
            let _ = interval.tick().await;
            let count = self.connection_count().await?;
            if f(count) {
                return Ok(count);
            }
        }
    }

    /// Poll until the player count satisfies the predicate.
    ///
    /// `poll_interval` controls how frequently the predicate is evaluated.
    pub async fn wait_for_player_count(
        &self,
        f: impl Fn(usize) -> bool,
        poll_interval: std::time::Duration,
    ) -> Result<usize, PresenceError> {
        let mut interval = time::interval(poll_interval);
        loop {
            let _ = interval.tick().await;
            let count = self.player_count().await?;
            if f(count) {
                return Ok(count);
            }
        }
    }

    /// Connection count
    pub async fn connection_count(&self) -> Result<usize, PresenceError> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(PresenceMessage::ConnectionCount { response_tx: tx })
            .await
            .map_err(|_| PresenceError::QueryChannelClosed)?;
        rx.await.map_err(|_| PresenceError::ResponseChannelClosed)
    }

    /// Player count
    pub async fn player_count(&self) -> Result<usize, PresenceError> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(PresenceMessage::PlayerCount { response_tx: tx })
            .await
            .map_err(|_| PresenceError::QueryChannelClosed)?;
        rx.await.map_err(|_| PresenceError::ResponseChannelClosed)
    }

    /// get all connections
    pub async fn connections(&self) -> Result<Vec<ConnectionInfo>, PresenceError> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(PresenceMessage::Connections { response_tx: tx })
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
        self.tx
            .send(PresenceMessage::Connection {
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
        self.tx
            .send(PresenceMessage::ConnectionByPlayer {
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
        self.tx
            .send(PresenceMessage::Players { response_tx: tx })
            .await
            .map_err(|_| PresenceError::QueryChannelClosed)?;
        rx.await.map_err(|_| PresenceError::ResponseChannelClosed)
    }

    /// get a player
    pub async fn player(&self, plid: &PlayerId) -> Result<Option<PlayerInfo>, PresenceError> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(PresenceMessage::Player {
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
        self.tx
            .send(PresenceMessage::LastKnownName {
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
        self.tx
            .send(PresenceMessage::LastKnownNames {
                unames: unames.into_iter().map(Into::into).collect(),
                response_tx: tx,
            })
            .await
            .map_err(|_| PresenceError::QueryChannelClosed)?;
        rx.await.map_err(|_| PresenceError::ResponseChannelClosed)
    }

    async fn send_command(
        &self,
        msg: PresenceMessage,
        rx: oneshot::Receiver<Result<(), PresenceError>>,
    ) -> Result<(), PresenceError> {
        self.tx
            .send(msg)
            .await
            .map_err(|_| PresenceError::QueryChannelClosed)?;
        rx.await.map_err(|_| PresenceError::ResponseChannelClosed)?
    }

    /// Kick a connection.
    pub async fn kick(&self, ucid: ConnectionId) -> Result<(), PresenceError> {
        let (response_tx, rx) = oneshot::channel();
        self.send_command(PresenceMessage::Kick { ucid, response_tx }, rx)
            .await
    }

    /// Ban a connection. `ban_days` of 0 = 12 hours.
    pub async fn ban(&self, ucid: ConnectionId, ban_days: u32) -> Result<(), PresenceError> {
        let (response_tx, rx) = oneshot::channel();
        self.send_command(
            PresenceMessage::Ban {
                ucid,
                ban_days,
                response_tx,
            },
            rx,
        )
        .await
    }

    /// Unban a player by LFS username.
    pub async fn unban(&self, uname: impl Into<String>) -> Result<(), PresenceError> {
        let (response_tx, rx) = oneshot::channel();
        self.send_command(
            PresenceMessage::Unban {
                uname: uname.into(),
                response_tx,
            },
            rx,
        )
        .await
    }

    /// Spec a connection.
    pub async fn spec(&self, ucid: ConnectionId) -> Result<(), PresenceError> {
        let (response_tx, rx) = oneshot::channel();
        self.send_command(PresenceMessage::Spec { ucid, response_tx }, rx)
            .await
    }

    /// Send a connection to the pit lane.
    pub async fn pitlane(&self, ucid: ConnectionId) -> Result<(), PresenceError> {
        let (response_tx, rx) = oneshot::channel();
        self.send_command(PresenceMessage::Pitlane { ucid, response_tx }, rx)
            .await
    }
}
