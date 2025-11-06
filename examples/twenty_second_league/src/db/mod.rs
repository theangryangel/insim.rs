mod game;
mod player;
mod score;

use std::path::PathBuf;

use anyhow::Result;
use rusqlite::Connection;
use rusqlite_migration::{M, Migrations};

#[derive(Debug, Clone)]
pub struct Repo {
    pub(crate) path: PathBuf,
}

impl Repo {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    fn open(&self) -> Result<Connection> {
        let conn = Connection::open(&self.path)?;
        conn.pragma_update(None, "foreign_keys", &"ON")?;
        Ok(conn)
    }

    pub fn migrate(&self) -> Result<()> {
        let mut conn = self.open()?;
        conn.pragma_update(None, "journal_mode", &"WAL")?;

        // TODO: record combos in database?
        // provide cli interface to add/remove combos?

        let migrations = Migrations::new(vec![
            M::up(include_str!("../migrations/20251106162526_up_bootstrap.sql"))
        ]);

        migrations.to_latest(&mut conn)?;
        Ok(())
    }
}
