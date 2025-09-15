//! Context mother fucker, do you speak it?

use std::{any::TypeId, collections::HashMap, fmt::Debug};

use insim::{
    identifiers::{ConnectionId, PlayerId}, insim::Mso, Packet
};
use tokio::sync::{broadcast, mpsc, oneshot};
use tokio_util::sync::CancellationToken;

use crate::{
    framework::Command,
    state::{ConnectionInfo, GameInfo, PlayerInfo},
    ui::node::UINode,
};

/// Framework PluginContext
#[derive(Debug)]
pub struct PluginContext<S>
where
    S: Send + Sync + Clone + Debug,
{
    /// events
    pub(crate) events: broadcast::Sender<Packet>,

    /// command sender
    pub(crate) commands: mpsc::Sender<Command>,

    /// cancellation
    pub(crate) cancellation_token: CancellationToken,

    /// user state
    pub user_state: S,
}

impl<S> PluginContext<S>
where
    S: Send + Sync + Clone + Debug,
{
    /// Wheres mah packets at?
    pub fn subscribe_to_packets(&self) -> broadcast::Receiver<Packet> {
        self.events.subscribe()
    }

    /// Send an insim packet
    pub async fn send_packet<P: Into<Packet>>(&self, packet: P) {
        // FIXME: handle error
        self.commands
            .send(Command::SendPacket(packet.into()))
            .await
            .unwrap();
    }

    /// Shortcut to send a command
    pub async fn send_command(&self, command: &str) {
        self.send_packet(Mso {
            msg: command.into(),
            ..Default::default()
        }).await;
    }

    /// Shortcut to send a command
    // TODO: make it pick the right packet type automatically.
    pub async fn send_message(&self, msg: &str) {
        self.send_packet(Mso {
            msg: msg.into(),
            ..Default::default()
        }).await;
    }

    /// Get a single player info
    pub async fn get_player(&self, player_id: PlayerId) -> Option<PlayerInfo> {
        let (response_tx, response_rx) = oneshot::channel();

        // FIXME: unwraps

        // Send the request
        let _ = self
            .commands
            .send(Command::GetPlayer(player_id, response_tx))
            .await
            .unwrap();

        response_rx.await.unwrap()
    }

    /// Get all player info
    pub async fn get_players(&self) -> HashMap<PlayerId, PlayerInfo> {
        let (response_tx, response_rx) = oneshot::channel();

        // FIXME: unwraps

        // Send the request
        let _ = self
            .commands
            .send(Command::GetPlayers(response_tx))
            .await
            .unwrap();

        response_rx.await.unwrap()
    }

    /// Get a single connection info
    pub async fn get_connection(&self, connection_id: ConnectionId) -> Option<ConnectionInfo> {
        let (response_tx, response_rx) = oneshot::channel();

        // FIXME: unwraps

        // Send the request
        let _ = self
            .commands
            .send(Command::GetConnection(connection_id, response_tx))
            .await
            .unwrap();

        response_rx.await.unwrap()
    }

    /// Get all player info
    pub async fn get_connections(&self) -> HashMap<ConnectionId, ConnectionInfo> {
        let (response_tx, response_rx) = oneshot::channel();

        // FIXME: unwraps

        // Send the request
        let _ = self
            .commands
            .send(Command::GetConnections(response_tx))
            .await
            .unwrap();

        response_rx.await.unwrap()
    }

    /// Get all Game info
    pub async fn get_game(&self) -> GameInfo {
        let (response_tx, response_rx) = oneshot::channel();

        // FIXME: unwraps

        // Send the request
        let _ = self
            .commands
            .send(Command::GetGame(response_tx))
            .await
            .unwrap();

        response_rx.await.unwrap()
    }

    /// Request Shutdown
    pub async fn shutdown(&self) {
        // FIXME: unwraps
        let _ = self.commands.send(Command::Shutdown).await.unwrap();
    }

    /// Update the UI
    pub async fn set_ui<T: Send + 'static>(&mut self, ucid: ConnectionId, node: UINode) {
        let _ = self
            .commands
            .send(Command::SetUi(TypeId::of::<T>(), ucid, node))
            .await
            .unwrap();
    }

    /// clear the ui
    pub async fn remove_ui<T: 'static>(&mut self, ucid: ConnectionId) {
        let _ = self
            .commands
            .send(Command::RemoveUi(TypeId::of::<T>(), ucid))
            .await
            .unwrap();
    }

    /// Wait for cancellation
    pub async fn abort(&self) {
        self.cancellation_token.cancelled().await
    }
}
