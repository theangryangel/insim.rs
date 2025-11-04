use anyhow::Result;
use rusqlite::params;

use super::Repo;

impl Repo {
    pub fn upsert_player(&self, uname: &str, pname: &str) -> Result<()> {
        let conn = self.open()?;
        let now = jiff::Timestamp::now().as_second();

        let _ = conn.execute(
            "INSERT INTO players (uname, pname, first_seen, last_seen)
             VALUES (?1, ?2, ?3, ?3)
             ON CONFLICT(uname) DO UPDATE SET pname = ?2, last_seen = ?3",
            params![uname, pname, now],
        )?;

        Ok(())
    }
}
