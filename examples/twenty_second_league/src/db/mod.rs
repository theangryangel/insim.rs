pub mod game;
pub mod models;
pub mod player;
pub mod repository;
pub mod score;

use std::path::Path;

use anyhow::Result;
use sqlx::{
    SqlitePool,
    sqlite::{SqliteConnectOptions, SqliteJournalMode},
};

pub async fn init(path: impl AsRef<Path>) -> Result<SqlitePool> {
    let options = SqliteConnectOptions::new()
        .filename(path.as_ref())
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal);

    let pool = SqlitePool::connect_with(options).await?;

    sqlx::migrate!().run(&pool).await?;

    Ok(pool)
}
