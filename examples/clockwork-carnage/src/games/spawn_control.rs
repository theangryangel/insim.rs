use std::collections::{HashMap, VecDeque};

use insim::{
    WithRequestId,
    builder::InsimTask,
    core::{
        heading::Heading,
        object::{ObjectCoordinate, insim::InsimCheckpointKind},
    },
    identifiers::{ConnectionId, RequestId},
    insim::{Jrr, JrrAction, JrrStartPosition, ObjectInfo, PmoAction, PmoFlags, TinyType},
};
use tokio::{sync::broadcast, task::JoinHandle};

const AXM_SCAN_REQUEST_ID: RequestId = RequestId(240);
const NPL_SYNC_REQUEST_ID: RequestId = RequestId(241);
const MIN_START_POSITIONS: usize = 4;

#[derive(Debug, Clone, Copy)]
struct SpawnPoint {
    xyz: ObjectCoordinate,
    heading: Heading,
}

#[derive(Debug)]
struct LayoutScan {
    spawn_points: VecDeque<SpawnPoint>,
    checkpoint1_count: usize,
    finish_count: usize,
}

#[derive(Debug, thiserror::Error)]
pub enum SpawnControlError {
    #[error("Insim: {0}")]
    Insim(#[from] insim::Error),
    #[error("Lost Insim handle")]
    InsimHandleLost,
    #[error("Layout requires at least {required} StartPosition objects, found {found}")]
    TooFewStartPositions { found: usize, required: usize },
    #[error("Layout requires at least one Insim Checkpoint1 object")]
    MissingCheckpoint1,
    #[error("Layout requires at least one Insim Finish object")]
    MissingFinish,
}

pub struct SpawnControlHandle {
    handle: JoinHandle<()>,
}

impl Drop for SpawnControlHandle {
    fn drop(&mut self) {
        self.handle.abort();
    }
}

pub async fn spawn(insim: InsimTask) -> Result<SpawnControlHandle, SpawnControlError> {
    let layout = scan_layout(&insim).await?;
    tracing::info!(
        "Spawn control initialised with {} start positions (cp1={}, finish={})",
        layout.spawn_points.len(),
        layout.checkpoint1_count,
        layout.finish_count
    );

    let mut packets = insim.subscribe();
    let handle = tokio::spawn(async move {
        let mut available = layout.spawn_points;
        let mut leases: HashMap<ConnectionId, SpawnPoint> = HashMap::new();

        let _ = insim
            .send(TinyType::Npl.with_request_id(NPL_SYNC_REQUEST_ID))
            .await;

        loop {
            match packets.recv().await {
                Ok(insim::Packet::Npl(npl)) => {
                    if npl.nump == 0 {
                        tracing::debug!(
                            "Ignoring join-request NPL for UCID {} (REQ_JOIN not in use)",
                            npl.ucid
                        );
                        continue;
                    }

                    let spawn_point = match leases.get(&npl.ucid).copied() {
                        Some(existing) => existing,
                        None => match available.pop_front() {
                            Some(new) => {
                                let _ = leases.insert(npl.ucid, new);
                                new
                            },
                            None => {
                                tracing::warn!(
                                    "No start position available for UCID {}, skipping teleport",
                                    npl.ucid
                                );
                                continue;
                            },
                        },
                    };

                    if let Err(err) = insim
                        .send(Jrr {
                            reqi: RequestId(0),
                            plid: npl.plid,
                            ucid: npl.ucid,
                            jrraction: JrrAction::Reset,
                            startpos: JrrStartPosition::Custom {
                                xyz: spawn_point.xyz,
                                heading: spawn_point.heading,
                            },
                        })
                        .await
                    {
                        tracing::warn!(
                            "Failed to teleport PLID {} (UCID {}): {}",
                            npl.plid,
                            npl.ucid,
                            err
                        );
                    }
                },
                Ok(insim::Packet::Cnl(cnl)) => {
                    if let Some(spawn_point) = leases.remove(&cnl.ucid) {
                        available.push_back(spawn_point);
                        tracing::debug!("Released spawn lease for UCID {}", cnl.ucid);
                    }
                },
                Ok(insim::Packet::Rst(_)) => {
                    let _ = insim
                        .send(TinyType::Npl.with_request_id(NPL_SYNC_REQUEST_ID))
                        .await;
                },
                Ok(_) => {},
                Err(broadcast::error::RecvError::Lagged(skipped)) => {
                    tracing::warn!("Spawn control lagged by {skipped} packets");
                },
                Err(broadcast::error::RecvError::Closed) => {
                    tracing::warn!("Spawn control stopping: packet stream closed");
                    break;
                },
            }
        }
    });

    Ok(SpawnControlHandle { handle })
}

async fn scan_layout(insim: &InsimTask) -> Result<LayoutScan, SpawnControlError> {
    let mut packets = insim.subscribe();

    insim
        .send(TinyType::Axm.with_request_id(AXM_SCAN_REQUEST_ID))
        .await?;

    let mut spawn_points = VecDeque::new();
    let mut checkpoint1_count = 0usize;
    let mut finish_count = 0usize;

    loop {
        match packets.recv().await {
            Ok(insim::Packet::Axm(axm))
                if axm.reqi == AXM_SCAN_REQUEST_ID
                    && matches!(axm.pmoaction, PmoAction::TinyAxm) =>
            {
                let is_final = axm.pmoflags.contains(PmoFlags::FILE_END);

                for object in axm.info {
                    match object {
                        ObjectInfo::PitStartPoint(start) => {
                            spawn_points.push_back(SpawnPoint {
                                xyz: start.xyz,
                                heading: start.heading,
                            });
                        },
                        ObjectInfo::InsimCheckpoint(checkpoint) => match checkpoint.kind {
                            InsimCheckpointKind::Checkpoint1 => checkpoint1_count += 1,
                            InsimCheckpointKind::Finish => finish_count += 1,
                            _ => {},
                        },
                        _ => {},
                    }
                }

                if is_final {
                    break;
                }
            },
            Ok(_) => {},
            Err(broadcast::error::RecvError::Lagged(skipped)) => {
                tracing::warn!("Spawn scan lagged by {skipped} packets");
            },
            Err(broadcast::error::RecvError::Closed) => {
                return Err(SpawnControlError::InsimHandleLost);
            },
        }
    }

    if spawn_points.len() < MIN_START_POSITIONS {
        return Err(SpawnControlError::TooFewStartPositions {
            found: spawn_points.len(),
            required: MIN_START_POSITIONS,
        });
    }

    if checkpoint1_count == 0 {
        return Err(SpawnControlError::MissingCheckpoint1);
    }

    if finish_count == 0 {
        return Err(SpawnControlError::MissingFinish);
    }

    Ok(LayoutScan {
        spawn_points,
        checkpoint1_count,
        finish_count,
    })
}
