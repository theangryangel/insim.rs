use anyhow::Result;
use rusqlite::params;

use super::Repo;

impl Repo {
    pub fn insert_player_score(
        &self,
        game_id: i64,
        round: u32,
        player_id: i64,
        points: i32,
        position: usize,
    ) -> Result<()> {
        let conn = self.open()?;

        let _ = conn.execute(
            "INSERT INTO round_score (game_id, round, player_id, points, position)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![game_id, round, player_id, points, position],
        )?;

        Ok(())
    }

    pub fn leaderboard(&self, game_id: i64, max: usize) {
        todo!()
    }
}
