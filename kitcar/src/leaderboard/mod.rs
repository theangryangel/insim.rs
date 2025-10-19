//! Leaderboard

use std::{collections::HashMap, hash::Hash};

/// Leaderboard/Scoreboard
#[derive(Debug, Default)]
pub struct Leaderboard<K: Clone + Hash + Eq + Default> {
    scores: HashMap<K, i32>,
}

impl<K: Clone + Hash + Eq + Default> Leaderboard<K> {
    /// New!
    pub fn new() -> Self {
        Self::default()
    }

    /// add to the score board for a player
    pub fn add_score(&mut self, plid: K, points: i32) -> i32 {
        *self
            .scores
            .entry(plid)
            .and_modify(|p| *p = p.saturating_add(points))
            .or_insert(points)
    }

    /// get the player score
    pub fn score(&self, plid: &K) -> Option<&i32> {
        self.scores.get(plid)
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
}
