//! Leaderboard

// TODO: the rest of the owl

use std::collections::HashMap;

use insim::{identifiers::PlayerId, Packet};
use tokio::sync::{mpsc, oneshot};

use crate::State;

/// Leaderboard/Scoreboard
#[derive(Debug, Default)]
pub struct Leaderboard {
    // FIXME: this is by playerid for now. we should probably use LFS uname..
    scores: HashMap<PlayerId, i32>,
}

impl Leaderboard {
    /// New!
    pub fn new() -> Self {
        Self::default()
    }

    /// add to the score board for a player
    pub fn add_player_score(&mut self, plid: &PlayerId, points: i32) -> i32 {
        self.scores
            .entry(*plid)
            .and_modify(|p| *p = p.saturating_add(points))
            .or_insert(points)
            .clone()
    }

    /// get the player score
    pub fn player_score(&self, plid: &PlayerId) -> Option<&i32> {
        self.scores.get(plid)
    }

    /// Ranking
    pub fn ranking(&self, truncate_to: Option<usize>) -> Vec<(PlayerId, i32)> {
        let mut ordered = self
            .scores
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect::<Vec<(PlayerId, i32)>>();

        ordered.sort_by(|a, b| b.1.cmp(&a.1));

        if let Some(max) = truncate_to {
            ordered.truncate(max);
        }

        ordered
    }

    /// Clear
    pub fn clear(&mut self) {
        self.scores.clear();
    }
}

#[derive(Debug)]
enum LeaderboardQuery {
    AddPlayerScore {
        plid: PlayerId,
        score: i32,
        response_tx: oneshot::Sender<i32>,
    },
    GetPlayerScore {
        plid: PlayerId,
        response_tx: oneshot::Sender<Option<i32>>,
    },
    GetRanking {
        truncate_to: Option<usize>,
        response_tx: oneshot::Sender<Vec<(PlayerId, i32)>>,
    },
    Clear,
}

#[derive(Debug, Clone)]
/// Handler for Leaderboard
pub struct LeaderboardHandle {
    query_tx: mpsc::Sender<LeaderboardQuery>,
}

impl LeaderboardHandle {
    /// Add to a player score
    pub async fn add_player_score(&self, plid: &PlayerId, score: i32) -> i32 {
        let (tx, rx) = oneshot::channel();
        let _ = self
            .query_tx
            .send(LeaderboardQuery::AddPlayerScore {
                plid: *plid,
                score,
                response_tx: tx,
            })
            .await;
        rx.await.unwrap_or_default()
    }

    /// Get a player score
    pub async fn player_score(&self, plid: &PlayerId) -> Option<i32> {
        let (tx, rx) = oneshot::channel();
        let _ = self
            .query_tx
            .send(LeaderboardQuery::GetPlayerScore {
                plid: *plid,
                response_tx: tx,
            })
            .await;
        rx.await.unwrap_or_default()
    }

    /// Get ranking
    pub async fn ranking(&self, truncate_to: Option<usize>) -> Vec<(PlayerId, i32)> {
        let (tx, rx) = oneshot::channel();
        let _ = self
            .query_tx
            .send(LeaderboardQuery::GetRanking {
                truncate_to,
                response_tx: tx,
            })
            .await;
        rx.await.unwrap_or_default()
    }

    /// Clear
    pub async fn clear(&self) {
        let _ = self.query_tx.send(LeaderboardQuery::Clear).await;
    }
}

impl State for Leaderboard {
    type H = LeaderboardHandle;

    fn update(&mut self, packet: &Packet) {
        match packet {
            Packet::Pll(pll) => {
                let _ = self.scores.remove(&pll.plid);
            },
            _ => {},
        }
    }

    fn spawn(insim: insim::builder::SpawnedHandle) -> Self::H {
        let (query_tx, mut query_rx) = mpsc::channel(Self::BROADCAST_CAPACITY);
        let _ = tokio::spawn(async move {
            let mut inner = Self::new();
            let mut packet_rx = insim.subscribe();

            loop {
                tokio::select! {
                    Ok(packet) = packet_rx.recv() => {
                        inner.update(&packet);
                    }
                    Some(query) = query_rx.recv() => {
                        match query {
                            LeaderboardQuery::GetPlayerScore { plid, response_tx } => {
                                let _ = response_tx.send(inner.player_score(&plid).cloned());
                            },
                            LeaderboardQuery::AddPlayerScore { plid, score, response_tx } => {
                                let _ = response_tx.send(inner.add_player_score(&plid, score));
                            },
                            LeaderboardQuery::Clear => {
                                inner.clear();
                            },
                            LeaderboardQuery::GetRanking { truncate_to, response_tx } => {
                                let _ = response_tx.send(inner.ranking(truncate_to));
                            },
                        }
                    }
                }
            }
        });

        LeaderboardHandle { query_tx }
    }
}
