use anyhow::Result;

use super::{Repo, models::Player};

impl Repo {
    pub async fn upsert_player(&self, uname: &str, pname: &str) -> Result<Player> {
        let now = jiff::Timestamp::now().to_string();

        // We use query_as! to return the Player struct.
        // RETURNING * is supported in SQLite 3.35+ (2021), which sqlx supports.
        let player = sqlx::query_as!(
            Player,
            r#"
            INSERT INTO player (uname, pname, first_seen_at, last_seen_at)
            VALUES (?, ?, ?, ?)
            ON CONFLICT(uname) DO UPDATE SET pname = ?, last_seen_at = ?
            RETURNING id, uname, pname, first_seen_at, last_seen_at
            "#,
            uname,
            pname,
            now,
            now,
            pname,
            now
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(player)
    }
}
