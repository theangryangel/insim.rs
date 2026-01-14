//! Scene to wait for minimum players. Useful to handle the situation where LFS is on the
//! multiplayer lobby screen. 

use insim::builder::SpawnedHandle;

use crate::presence;
use super::{Scene, SceneResult, SceneError};

/// Wait for minimum players to connect
#[derive(Clone)]
pub struct WaitForPlayers {
    /// Insim handle
    pub insim: SpawnedHandle,
    /// Presence handle
    pub presence: presence::Presence,
    /// Minimum number of players required. It should include the dedicated server itself.
    pub min_players: usize,
}

impl Scene for WaitForPlayers {
    type Output = ();

    async fn run(mut self) -> Result<SceneResult<()>, SceneError> {
        tracing::info!("Waiting for {} players...", self.min_players);
        let mut packets = self.insim.subscribe();

        loop {
            tokio::select! {
                packet = packets.recv() => {
                    if let insim::Packet::Ncn(ncn) = packet.map_err(|_| SceneError::InsimHandleLost)? {
                        self.insim.send_message("Waiting for players", ncn.ucid).await?;
                    }
                }
                _ = self.presence.wait_for_connection_count(|val| *val >= self.min_players) => {
                    tracing::info!("Got minimum player count!");
                    return Ok(SceneResult::Continue(()));
                }
            }
        }
    }
}
