use std::time::Duration;

use anyhow::Result;
use rusqlite::params;

use super::Repo;
use crate::db::game::EventId;

impl Repo {
    pub fn insert_player_scores(
        &self,
        game_id: EventId,
        round: u32,
        batch: Vec<(String, i32, usize, Duration)>,
    ) -> Result<()> {
        let mut conn = self.open()?;

        let tx = conn.transaction()?;

        for (uname, points, position, delta) in batch.into_iter() {
            let player_id: i64 = tx.query_row(
                "SELECT id FROM player WHERE uname = ?1",
                params![uname],
                |row| row.get(0),
            )?;

            let _ = tx.execute(
                "INSERT INTO result (event_id, round, player_id, position, points, delta)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    game_id.0,
                    round,
                    player_id,
                    position,
                    points,
                    // XXX: If this overflows, then something horrible has happened. this is good
                    // enough for now.
                    delta.as_millis() as i64
                ],
            )?;
        }

        tx.commit()?;

        Ok(())
    }

    pub fn leaderboard(&self, game_id: EventId, max: usize) -> Result<Vec<(String, i32, i64)>> {
        let conn = self.open()?;
        let leaderboard = conn.prepare(
            "SELECT pname, total_points, position FROM leaderboard WHERE event_id = ? ORDER BY position DESC LIMIT ?",
        )?.query_map(params![game_id.0, max], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))

        })?.collect::<Result<Vec<_>, _>>();

        Ok(leaderboard?)
    }
}
