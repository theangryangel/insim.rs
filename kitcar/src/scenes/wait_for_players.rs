//! Scene to wait for minimum players. Useful to handle the situation where LFS is on the
//! multiplayer lobby screen.

use insim::builder::InsimTask;

use super::{FromContext, Scene, SceneError, SceneResult};
use crate::presence;

/// Wait for minimum players to connect
#[derive(Clone)]
pub struct WaitForPlayers {
    /// Minimum number of players required. It should include the dedicated server itself.
    pub min_players: usize,
}

impl<Ctx> Scene<Ctx> for WaitForPlayers
where
    InsimTask: FromContext<Ctx>,
    presence::Presence: FromContext<Ctx>,
    Ctx: Sync,
{
    type Output = ();

    async fn run(self, ctx: &Ctx) -> Result<SceneResult<()>, SceneError> {
        let insim = InsimTask::from_context(ctx);
        let presence = presence::Presence::from_context(ctx);

        tracing::info!("Waiting for {} players...", self.min_players);
        let mut packets = insim.subscribe();

        loop {
            tokio::select! {
                packet = packets.recv() => {
                    if let insim::Packet::Ncn(ncn) = packet.map_err(|_| SceneError::InsimHandleLost)? {
                        insim.send_message("Waiting for players", ncn.ucid).await?;
                    }
                }
                res = presence.wait_for_connection_count(|val| val >= self.min_players, std::time::Duration::from_millis(500)) => {
                    let _ = res.map_err(|cause| SceneError::Custom {
                        scene: "wait_for_players::wait_for_connection_count",
                        cause: Box::new(cause),
                    })?;
                    tracing::info!("Got minimum player count!");
                    return Ok(SceneResult::Continue(()));
                }
            }
        }
    }
}
