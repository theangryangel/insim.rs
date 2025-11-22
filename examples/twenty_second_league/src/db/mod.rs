pub mod game;
pub mod models;
pub mod player;
pub mod score;

use std::path::Path;

use anyhow::Result;
use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteJournalMode},
    SqlitePool,
};

#[derive(Debug, Clone)]
pub struct Repo {
    pub(crate) pool: SqlitePool,
}

impl Repo {
    pub async fn new(path: impl AsRef<Path>) -> Result<Self> {
        let options = SqliteConnectOptions::new()
            .filename(path.as_ref())
            .create_if_missing(true)
            .journal_mode(SqliteJournalMode::Wal);

        let pool = SqlitePool::connect_with(options).await?;

        Ok(Self { pool })
    }

    pub async fn migrate(&self) -> Result<()> {
        sqlx::migrate!().run(&self.pool).await?;
        Ok(())
    }
}
