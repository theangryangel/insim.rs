use std::time::Duration;

use anyhow::Result;

use super::{Repo, models::LeaderboardEntry};
use crate::db::game::EventId;

impl Repo {
    pub async fn insert_player_scores(
        &self,
        game_id: EventId,
        round: u32,
        batch: Vec<(String, i32, usize, Duration)>,
    ) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        for (uname, points, position, delta) in batch.into_iter() {
            let player_id = sqlx::query_scalar!(
                "SELECT id FROM player WHERE uname = ?",
                uname
            )
            .fetch_one(&mut *tx)
            .await?;

            let delta_ms = delta.as_millis() as i64;
            // Schema says position is INTEGER (i64/i32).
            let position_i64 = position as i64;

            let _ = sqlx::query!(
                "INSERT INTO result (event_id, round, player_id, position, points, delta)
                 VALUES (?, ?, ?, ?, ?, ?)",
                game_id.0,
                round,
                player_id,
                position_i64,
                points,
                delta_ms
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        Ok(())
    }

    pub async fn leaderboard(&self, game_id: EventId, max: usize) -> Result<Vec<LeaderboardEntry>> {
        let max_i64 = max as i64;
        let results = sqlx::query_as!(
            LeaderboardEntry,
            r#"
            SELECT pname,
                   total_points as "total_points!: i64",
                   position as "position!: i64"
            FROM leaderboard
            WHERE event_id = ?
            ORDER BY position ASC
            LIMIT ?
            "#,
            game_id.0,
            max_i64
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(results)
    }
}
