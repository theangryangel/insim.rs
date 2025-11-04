use anyhow::Result;
use rusqlite::params;

use super::Repo;

impl Repo {
    pub fn new_game(&self, combo: &str) -> Result<i64> {
        let conn = self.open()?;
        let now = jiff::Timestamp::now();

        // TODO: combo shouldn't be a str?
        // Question, do we want to store combos in the database? eghasdj.

        let id: i64 = conn.query_row(
            "INSERT INTO game (combo, started_at)
             VALUES (?1, ?2)
             RETURNING id",
            params![combo, now],
            |row| row.get(0),
        )?;

        Ok(id)
    }

    pub fn complete_game(&self, game_id: i64) -> Result<()> {
        let conn = self.open()?;
        let now = jiff::Timestamp::now();
        let _ = conn.execute(
            "UPDATE game SET completed_at = ?1 WHERE id = ?2",
            params![now, game_id],
        )?;

        Ok(())
    }
}
