use anyhow::Result;
use kitcar::combos::Combo;
use rusqlite::params;

use crate::combo::ComboExt;

use super::Repo;

pub struct EventId(pub i64);

impl Repo {
    pub fn new_event(&self, combo: &Combo<ComboExt>) -> Result<EventId> {
        let conn = self.open()?;
        let now = jiff::Timestamp::now();

        // TODO: combo shouldn't be a str?
        // Question, do we want to store combos in the database? eghasdj.

        let id: i64 = conn.query_row(
            "INSERT INTO event (started_at, name, track, layout, target_time, restart_after, rounds)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             RETURNING id",
            params![
                now, 
                combo.extensions().name, 
                combo.track().code(), 
                combo.layout(), 
                combo.extensions().target_time.to_string(), 
                combo.extensions().restart_after.to_string(), 
                combo.extensions().rounds, 
            ],
            |row| row.get(0),
        )?;

        Ok(EventId(id))
    }

    pub fn complete_event(&self, event_id: EventId) -> Result<()> {
        let conn = self.open()?;
        let now = jiff::Timestamp::now();
        let _ = conn.execute(
            "UPDATE game SET completed_at = ?1 WHERE id = ?2",
            params![now, event_id.0],
        )?;

        Ok(())
    }
}
