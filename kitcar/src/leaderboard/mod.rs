//! Leaderboard

use std::{collections::HashMap, hash::Hash};

use tokio::sync::{mpsc, oneshot};

/// Leaderboard/Scoreboard
#[derive(Debug, Default)]
pub struct Leaderboard<K: Clone + Hash + Eq + Send + Sync> {
    scores: HashMap<K, i32>,
}

impl<K: Clone + Hash + Eq + Send + Sync> Leaderboard<K> {
    /// New!
    pub fn new() -> Self {
        Self {
            scores: HashMap::new(),
        }
    }

    /// add to the score board for a player
    pub fn add_score(&mut self, key: K, points: i32) -> i32 {
        *self
            .scores
            .entry(key)
            .and_modify(|p| *p = p.saturating_add(points))
            .or_insert(points)
    }

    /// get the player score
    pub fn score(&self, key: &K) -> Option<&i32> {
        self.scores.get(key)
    }

    /// Ranking
    pub fn top_n_ranking(&self, truncate_to: Option<usize>) -> Vec<(K, i32)> {
        let mut ordered = self
            .scores
            .iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect::<Vec<(K, i32)>>();

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

    /// Spawn a background instance of Leaderboard and return a handle so that we can query it
    pub fn spawn(capacity: usize) -> LeaderboardHandle<K> {
        let (query_tx, mut query_rx) = mpsc::channel::<LeaderboardQuery<K>>(capacity);
        let _handle = tokio::spawn(async move {
            let mut inner = Self::new();

            while let Some(query) = query_rx.recv().await {
                match query {
                    LeaderboardQuery::GetScore { key, response_tx } => {
                        let _ = response_tx.send(inner.score(&key).cloned());
                    },
                    LeaderboardQuery::AddScore {
                        key,
                        points,
                        response_tx,
                    } => {
                        let _ = response_tx.send(inner.add_score(key, points));
                    },
                    LeaderboardQuery::Clear => {
                        inner.clear();
                    },
                    LeaderboardQuery::TopNRanking {
                        truncate_to,
                        response_tx,
                    } => {
                        let _ = response_tx.send(inner.top_n_ranking(truncate_to));
                    },
                }
            }
        });

        LeaderboardHandle { query_tx }
    }
}

#[derive(Debug)]
enum LeaderboardQuery<K: Clone + Hash + Eq + Send + Sync> {
    AddScore {
        key: K,
        points: i32,
        response_tx: oneshot::Sender<i32>,
    },
    GetScore {
        key: K,
        response_tx: oneshot::Sender<Option<i32>>,
    },
    TopNRanking {
        truncate_to: Option<usize>,
        response_tx: oneshot::Sender<Vec<(K, i32)>>,
    },
    Clear,
}

#[derive(Debug, Clone)]
/// Handler for Leaderboard
pub struct LeaderboardHandle<K: Clone + Hash + Eq + Send + Sync + 'static> {
    query_tx: mpsc::Sender<LeaderboardQuery<K>>,
}

impl<K: Clone + Hash + Eq + Send + Sync> LeaderboardHandle<K> {
    /// Add to a player score
    pub async fn add_score(&self, key: K, score: i32) -> i32 {
        let (tx, rx) = oneshot::channel();
        let _ = self
            .query_tx
            .send(LeaderboardQuery::AddScore {
                key,
                points: score,
                response_tx: tx,
            })
            .await;
        rx.await.unwrap_or_default() // FIXME: handle err
    }

    /// Get a player score
    pub async fn score(&self, key: K) -> Option<i32> {
        let (tx, rx) = oneshot::channel();
        let _ = self
            .query_tx
            .send(LeaderboardQuery::GetScore {
                key,
                response_tx: tx,
            })
            .await;
        rx.await.unwrap_or_default() // FIXME: handle err
    }

    /// Get ranking
    pub async fn top_n_ranking(&self, truncate_to: Option<usize>) -> Vec<(K, i32)> {
        let (tx, rx) = oneshot::channel();
        let _ = self
            .query_tx
            .send(LeaderboardQuery::TopNRanking {
                truncate_to,
                response_tx: tx,
            })
            .await;
        rx.await.unwrap_or_default() // FIXME: handle err
    }

    /// Clear
    pub async fn clear(&self) {
        let _ = self.query_tx.send(LeaderboardQuery::Clear).await;
    }
}
