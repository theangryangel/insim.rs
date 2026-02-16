use std::{collections::HashMap, sync::Arc};

pub type LeaderboardRanking = Arc<[(String, u32)]>;

#[derive(Debug, Clone, Default)]
pub struct Leaderboard {
    scores: HashMap<String, u32>,
    ranked: LeaderboardRanking,
}

impl Leaderboard {
    pub fn add_points(&mut self, uname: String, points: u32) {
        *self.scores.entry(uname).or_insert(0) += points;
    }

    pub fn rank(&mut self) {
        let mut ranked: Vec<_> = self.scores.iter().map(|(k, v)| (k.clone(), *v)).collect();
        ranked.sort_by(|a, b| b.1.cmp(&a.1));
        self.ranked = ranked.into();
    }

    pub fn ranking(&self) -> LeaderboardRanking {
        Arc::clone(&self.ranked)
    }
}
