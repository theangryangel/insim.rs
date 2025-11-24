use anyhow::Result;
use kitcar::combos::Combo;

use super::{Repo, models::Event};
use crate::combo::ComboExt;

#[derive(Debug, Clone, Copy)]
pub struct EventId(pub i64);

impl Repo {
    pub async fn new_event(&self, combo: &Combo<ComboExt>) -> Result<Event> {
        let now = jiff::Timestamp::now().to_string();
        let target = combo.extensions().target_time.to_string();
        let restart = combo.extensions().restart_after.to_string();
        let rounds = combo.extensions().rounds;
        let track = combo.track().code();
        let layout = combo.layout();
        let name = combo.extensions().name.clone();

        let event = sqlx::query_as::<_, Event>(
            r#"
            INSERT INTO event (started_at, name, track, layout, target_time, restart_after, rounds)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            RETURNING id, started_at, completed_at, name, track, layout, target_time, restart_after, rounds
            "#
        )
        .bind(now)
        .bind(name)
        .bind(track)
        .bind(layout)
        .bind(target)
        .bind(restart)
        .bind(rounds)
        .fetch_one(&self.pool)
        .await?;

        Ok(event)
    }

    pub async fn complete_event(&self, event_id: EventId) -> Result<()> {
        let now = jiff::Timestamp::now().to_string();
        sqlx::query("UPDATE event SET completed_at = ? WHERE id = ?")
            .bind(now)
            .bind(event_id.0)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}
