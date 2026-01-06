use insim::builder::SpawnedHandle;
use kitcar::presence::PresenceHandle;

use crate::Scene;

pub struct WaitForPlayers
{
    insim: SpawnedHandle,
    presence: PresenceHandle,
    min_players: usize,
}

impl WaitForPlayers {
    pub fn new(insim: SpawnedHandle, presence: PresenceHandle, min_players: usize) -> Self {
        Self {
            insim, presence, min_players
        }
    }

    pub async fn poll<L, C>(&mut self, inner: L, ctx: C) -> L::Output
    where 
        L: Scene<C> + Clone,
        C: Send + Sync + Clone + 'static {
        loop {
            tracing::info!("Waiting for players...");
            let min = self.min_players;
            let mut packets = self.insim.subscribe();
            loop {
                tokio::select! {
                    packet = packets.recv() => match packet {
                        Ok(insim::Packet::Ncn(ncn)) => {
                            tracing::info!("Waiting for players...");
                            let _ = self.insim.send_message("Waiting for players", ncn.ucid).await.expect("Unhandled error");
                        },
                        _ => {

                        }
                    },
                    _ = self.presence.wait_for_player_count(|val| *val >= min) => {
                        break;
                    }
                }
            }

            let mut h = tokio::spawn({
                let mut inst = inner.clone();
                let ctx = ctx.clone();
                async move { inst.poll(ctx).await }
            });

            tokio::select! {
                result = &mut h => {
                    tracing::info!("{:?}", result);
                    match result {
                        Ok(result) => return result,
                        Err(e) => {
                            if e.is_cancelled() {
                                tracing::warn!("Game was cancelled.");
                            } else {
                                tracing::error!("Panicked! {:?}", e);
                            }
                            // If it crashed, we restart the loop
                            continue; 
                        }
                    }
                },
                _ = self.presence.wait_for_player_count(|val| *val < min) => {
                    h.abort();
                    tracing::error!("out of players. going back to the start");
                    // run out of players. go back to the start
                    continue
                }
            }
        }
    }
}

