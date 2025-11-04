use anyhow::Result;
use rusqlite::params;

use super::Repo;

impl Repo {
    pub fn insert_player_score(
        &self,
        game_id: i64,
        round: u32,
        uname: &str,
        points: i32,
        position: usize,
        delta: u64,
    ) -> Result<()> {
        let conn = self.open()?;

        let player_id: i64 = conn.query_row(
            "SELECT id FROM player WHERE uname = ?1",
            params![uname],
            |row| row.get(0),
        )?;

        let _ = conn.execute(
            "INSERT INTO round_score (game_id, round, player_id, points, position, delta)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![game_id, round, player_id, points, position, delta],
        )?;

        Ok(())
    }

    pub fn leaderboard(&self, game_id: i64, max: usize) {
        todo!()
    }
}
