use insim::{builder::SpawnedHandle, identifiers::ConnectionId};
use kitcar::{game::GameHandle, presence::PresenceHandle};

use crate::{Scene, scene::SceneError};

pub struct WaitForPlayers {
    insim: SpawnedHandle,
    presence: PresenceHandle,
    min_players: usize,
    inf: bool,
}

impl WaitForPlayers {
    pub fn new(insim: SpawnedHandle, presence: PresenceHandle, min_players: usize) -> Self {
        Self {
            insim,
            presence,
            min_players,
            inf: true,
        }
    }

    pub async fn poll<L, C>(&mut self, inner: L, ctx: C) -> Result<L::Output, L::Error>
    where
        L: Scene<C> + Clone,
        C: Send + Sync + Clone + 'static,
    {
        loop {
            tracing::info!("Waiting for players...");
            let _ = self
                .insim
                .send_message("Waiting for players", ConnectionId::ALL)
                .await
                .expect("Unhandled error");

            let min = self.min_players;
            let mut packets = self.insim.subscribe();
            loop {
                tokio::select! {
                    packet = packets.recv() => {
                        // FIXME expect
                        if let insim::Packet::Ncn(ncn) = packet.expect("FIXME: packet receiver failed") {
                            tracing::info!("Waiting for players...");
                            let _ = self.insim.send_message("Waiting for players", ncn.ucid).await.expect("Unhandled error");
                        }
                    },
                    _ = self.presence.wait_for_connection_count(|val| *val >= min) => {
                        tracing::info!("Got minimum player count!");
                        break;
                    }
                }
            }
            drop(packets);

            tracing::info!("Booting up inner");

            let mut h = tokio::spawn({
                let mut inst = inner.clone();
                let ctx = ctx.clone();
                async move { inst.poll(ctx).await }
            });

            tokio::select! {
                result = &mut h => {
                    tracing::info!("{:?}", result);
                    match result {
                        Ok(Ok(result)) => {
                            tracing::info!("Inner completed: {:?}", result);
                            if self.inf {
                                continue;
                            } else {
                                return Ok(result);
                            }
                        },
                        Ok(Err(e)) => {
                            if e.is_recoverable() && self.inf {
                                // If it crashed, we restart the loop
                                continue;
                            } else {
                                return Err(e);
                            }
                        },
                        Err(e) => {
                            if e.is_cancelled() {
                                tracing::warn!("Game was cancelled.");
                            } else {
                                tracing::error!("Panicked! {:?}", e);
                            }
                            continue;
                        }
                    }
                },
                _ = self.presence.wait_for_connection_count(|val| *val < min) => {
                    h.abort();
                    tracing::error!("out of connections. going back to the start");
                    // run out of players. go back to the start
                    continue
                },
            }
        }
    }
}
