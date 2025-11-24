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
            let player_id: i64 = sqlx::query_scalar("SELECT id FROM player WHERE uname = ?")
                .bind(&uname)
                .fetch_one(&mut *tx)
                .await?;

            let delta_ms = delta.as_millis() as i64;
            // Schema says position is INTEGER (i64/i32).
            let position_i64 = position as i64;

            sqlx::query(
                "INSERT INTO result (event_id, round, player_id, position, points, delta)
                 VALUES (?, ?, ?, ?, ?, ?)",
            )
            .bind(game_id.0)
            .bind(round)
            .bind(player_id)
            .bind(position_i64)
            .bind(points)
            .bind(delta_ms)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        Ok(())
    }

    pub async fn leaderboard(&self, game_id: EventId, max: usize) -> Result<Vec<LeaderboardEntry>> {
        let max_i64 = max as i64;
        let results = sqlx::query_as::<_, LeaderboardEntry>(
            r#"
            SELECT pname,
                   total_points,
                   position
            FROM leaderboard
            WHERE event_id = ?
            ORDER BY position ASC
            LIMIT ?
            "#,
        )
        .bind(game_id.0)
        .bind(max_i64)
        .fetch_all(&self.pool)
        .await?;

        Ok(results)
    }
}
