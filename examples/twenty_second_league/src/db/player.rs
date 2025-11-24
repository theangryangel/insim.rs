use anyhow::Result;
use sqlx::Executor;

use super::{
    models::Player,
    repository::{Repository, RepositoryCreate},
};

impl Repository for Player {
    type Model = Player;
    type Id = i64;
}

impl RepositoryCreate for Player {}

impl Player {
    pub async fn upsert<'e, E>(executor: E, uname: &str, pname: &str) -> Result<Self>
    where
        E: Executor<'e, Database = sqlx::Sqlite>,
    {
        let now = jiff::Timestamp::now().to_string();

        let player = sqlx::query_as::<_, Player>(
            r#"
            INSERT INTO player (uname, pname, first_seen_at, last_seen_at)
            VALUES (?, ?, ?, ?)
            ON CONFLICT(uname) DO UPDATE SET pname = ?, last_seen_at = ?
            RETURNING id, uname, pname, first_seen_at, last_seen_at
            "#,
        )
        .bind(uname)
        .bind(pname)
        .bind(&now)
        .bind(&now)
        .bind(pname)
        .bind(&now)
        .fetch_one(executor)
        .await?;

        Ok(player)
    }
}
