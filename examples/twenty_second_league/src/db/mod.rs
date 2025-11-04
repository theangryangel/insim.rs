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
            M::up(
                "CREATE TABLE game (
                    id INTEGER PRIMARY KEY,
                    combo TEXT NOT NULL,
                    started_at TEXT NOT NULL,
                    completed_at TEXT
                )",
            ),
            M::up(
                "CREATE TABLE player (
                    id INTEGER PRIMARY KEY,
                    uname TEXT NOT NULL,
                    pname TEXT NOT NULL,
                    first_seen TEXT NOT NULL,
                    last_seen TEXT NOT NULL
                )",
            ),
            M::up(
                "CREATE TABLE round_score (
                    game_id INTEGER NOT NULL,
                    round INTEGER NOT NULL,
                    player_id INTEGER NOT NULL,
                    points INTEGER NOT NULL,
                    delta INTEGER NOT NULL,
                    PRIMARY KEY (game_id, round, player_id),
                    FOREIGN KEY (game_id) REFERENCES game(id) ON DELETE CASCADE,
                    FOREIGN KEY (player_id) REFERENCES player(id)
                )",
            ),
        ]);

        migrations.to_latest(&mut conn)?;
        Ok(())
    }
}
