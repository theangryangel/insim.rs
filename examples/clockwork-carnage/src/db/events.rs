use insim::core::{track::Track, vehicle::Vehicle};
use sqlx::{Row, types::Json};

use super::{Event, EventMode, Pool, Timestamp};

pub async fn has_scheduling_overlap(
    pool: &Pool,
    start: Timestamp,
    end: Timestamp,
    exclude_id: Option<i64>,
) -> Result<bool, sqlx::Error> {
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM events
         WHERE status IN ('PENDING', 'ACTIVE')
         AND (? IS NULL OR id != ?)
         AND scheduled_at < ?
         AND (scheduled_end_at IS NULL OR scheduled_end_at > ?)",
    )
    .bind(exclude_id)
    .bind(exclude_id)
    .bind(end)
    .bind(start)
    .fetch_one(pool)
    .await?;
    Ok(count > 0)
}

pub struct CreateMetronomeParams {
    pub track: Track,
    pub layout: String,
    pub target_ms: i64,
    pub name: Option<String>,
    pub description: Option<String>,
    pub scheduled_at: Option<Timestamp>,
    pub scheduled_end_at: Option<Timestamp>,
}

pub struct CreateShortcutParams {
    pub track: Track,
    pub layout: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub scheduled_at: Option<Timestamp>,
    pub scheduled_end_at: Option<Timestamp>,
}

pub struct CreateBombParams {
    pub track: Track,
    pub layout: String,
    pub checkpoint_timeout_secs: i64,
    pub checkpoint_penalty_ms: i64,
    pub collision_max_penalty_ms: i64,
    pub name: Option<String>,
    pub description: Option<String>,
    pub scheduled_at: Option<Timestamp>,
    pub scheduled_end_at: Option<Timestamp>,
}

pub struct UpdateEventParams<'a> {
    pub track: Track,
    pub layout: &'a str,
    pub name: Option<&'a str>,
    pub description: Option<&'a str>,
    pub scheduled_at: Option<Timestamp>,
    pub scheduled_end_at: Option<Timestamp>,
    pub writeup: Option<&'a str>,
}

pub async fn all_events(pool: &Pool) -> Result<Vec<Event>, sqlx::Error> {
    sqlx::query_as(
        "SELECT id, mode, status, track, layout, scheduled_at, scheduled_end_at, name, description, writeup, allowed_vehicles
         FROM events ORDER BY id DESC",
    )
    .fetch_all(pool)
    .await
}

pub async fn get_event(pool: &Pool, id: i64) -> Result<Option<Event>, sqlx::Error> {
    sqlx::query_as(
        "SELECT id, mode, status, track, layout, scheduled_at, scheduled_end_at, name, description, writeup, allowed_vehicles
         FROM events WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn active_event(pool: &Pool) -> Result<Option<Event>, sqlx::Error> {
    sqlx::query_as(
        "SELECT id, mode, status, track, layout, scheduled_at, scheduled_end_at, name, description, writeup, allowed_vehicles
         FROM events WHERE status = 'ACTIVE'
         LIMIT 1",
    )
    .fetch_optional(pool)
    .await
}

pub async fn upcoming_events(pool: &Pool) -> Result<Vec<Event>, sqlx::Error> {
    sqlx::query_as(
        "SELECT id, mode, status, track, layout, scheduled_at, scheduled_end_at, name, description, writeup, allowed_vehicles
         FROM events WHERE status = 'PENDING' ORDER BY id DESC LIMIT 10",
    )
    .fetch_all(pool)
    .await
}

pub async fn next_due_event(pool: &Pool) -> Result<Option<Event>, sqlx::Error> {
    let now = Timestamp::now();
    sqlx::query_as(
        "SELECT id, mode, status, track, layout, scheduled_at, scheduled_end_at,
                name, description, writeup, allowed_vehicles
         FROM events
         WHERE status = 'PENDING'
           AND scheduled_at IS NOT NULL
           AND scheduled_at <= ?
         ORDER BY scheduled_at ASC
         LIMIT 1",
    )
    .bind(now)
    .fetch_optional(pool)
    .await
}

pub async fn next_scheduled_event(pool: &Pool) -> Result<Option<(Event, i64)>, sqlx::Error> {
    let now = Timestamp::now();
    let event = sqlx::query_as::<_, Event>(
        "SELECT id, mode, status, track, layout, scheduled_at, scheduled_end_at,
                name, description, writeup, allowed_vehicles
         FROM events
         WHERE status = 'PENDING'
           AND scheduled_at IS NOT NULL
           AND scheduled_at > ?
         ORDER BY scheduled_at ASC
         LIMIT 1",
    )
    .bind(now)
    .fetch_optional(pool)
    .await?;

    let Some(event) = event else { return Ok(None) };

    let secs = event.scheduled_at.unwrap().as_second() - now.as_second();

    Ok(Some((event, secs)))
}

pub async fn create_metronome_event(
    pool: &Pool,
    p: &CreateMetronomeParams,
) -> Result<i64, sqlx::Error> {
    let row = sqlx::query(
        "INSERT INTO events (mode, track, layout, created_at, name, description, scheduled_at, scheduled_end_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?) RETURNING id",
    )
    .bind(Json(EventMode::Metronome { target_ms: p.target_ms }))
    .bind(p.track.to_string())
    .bind(&p.layout)
    .bind(Timestamp::now())
    .bind(p.name.as_deref())
    .bind(p.description.as_deref())
    .bind(p.scheduled_at)
    .bind(p.scheduled_end_at)
    .fetch_one(pool)
    .await?;
    Ok(row.get("id"))
}

pub async fn create_shortcut_event(
    pool: &Pool,
    p: &CreateShortcutParams,
) -> Result<i64, sqlx::Error> {
    let row = sqlx::query(
        "INSERT INTO events (mode, track, layout, created_at, name, description, scheduled_at, scheduled_end_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?) RETURNING id",
    )
    .bind(Json(EventMode::Shortcut))
    .bind(p.track.to_string())
    .bind(&p.layout)
    .bind(Timestamp::now())
    .bind(p.name.as_deref())
    .bind(p.description.as_deref())
    .bind(p.scheduled_at)
    .bind(p.scheduled_end_at)
    .fetch_one(pool)
    .await?;
    Ok(row.get("id"))
}

pub async fn create_bomb_event(pool: &Pool, p: &CreateBombParams) -> Result<i64, sqlx::Error> {
    let row = sqlx::query(
        "INSERT INTO events (mode, track, layout, created_at, name, description, scheduled_at, scheduled_end_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?) RETURNING id",
    )
    .bind(Json(EventMode::Bomb {
        checkpoint_timeout_secs: p.checkpoint_timeout_secs,
        checkpoint_penalty_ms: p.checkpoint_penalty_ms,
        collision_max_penalty_ms: p.collision_max_penalty_ms,
    }))
    .bind(p.track.to_string())
    .bind(&p.layout)
    .bind(Timestamp::now())
    .bind(p.name.as_deref())
    .bind(p.description.as_deref())
    .bind(p.scheduled_at)
    .bind(p.scheduled_end_at)
    .fetch_one(pool)
    .await?;
    Ok(row.get("id"))
}

pub async fn switch_event(pool: &Pool, event_id: i64) -> Result<(), sqlx::Error> {
    let now = Timestamp::now();
    let mut tx = pool.begin().await?;
    let _ = sqlx::query(
        "UPDATE events SET status = 'COMPLETED', ended_at = ? WHERE status = 'ACTIVE' AND id != ?",
    )
    .bind(now)
    .bind(event_id)
    .execute(&mut *tx)
    .await?;

    let _ = sqlx::query(
        "UPDATE events SET status = 'ACTIVE', started_at = ? WHERE id = ? AND status = 'PENDING'",
    )
    .bind(now)
    .bind(event_id)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(())
}

pub async fn complete_event(pool: &Pool, event_id: i64) -> Result<(), sqlx::Error> {
    let now = Timestamp::now();
    let _ = sqlx::query(
        "UPDATE events SET status = 'COMPLETED', ended_at = ? WHERE id = ?",
    )
    .bind(now)
    .bind(event_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn cancel_event(pool: &Pool, event_id: i64) -> Result<(), sqlx::Error> {
    let now = Timestamp::now();
    let _ = sqlx::query(
        "UPDATE events SET status = 'CANCELLED', ended_at = ? WHERE id = ?",
    )
    .bind(now)
    .bind(event_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn update_event(
    pool: &Pool,
    id: i64,
    p: &UpdateEventParams<'_>,
) -> Result<(), sqlx::Error> {
    let _ = sqlx::query(
        "UPDATE events SET track = ?, layout = ?, name = ?, description = ?, scheduled_at = ?, scheduled_end_at = ?, writeup = ? WHERE id = ?",
    )
    .bind(p.track.to_string())
    .bind(p.layout)
    .bind(p.name)
    .bind(p.description)
    .bind(p.scheduled_at)
    .bind(p.scheduled_end_at)
    .bind(p.writeup)
    .bind(id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn update_metronome_settings(
    pool: &Pool,
    event_id: i64,
    target_ms: i64,
) -> Result<(), sqlx::Error> {
    let _ = sqlx::query("UPDATE events SET mode = json_set(mode, '$.target_ms', ?) WHERE id = ?")
        .bind(target_ms)
        .bind(event_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn update_bomb_settings(
    pool: &Pool,
    event_id: i64,
    checkpoint_timeout_secs: i64,
) -> Result<(), sqlx::Error> {
    let _ = sqlx::query(
        "UPDATE events SET mode = json_set(mode, '$.checkpoint_timeout_secs', ?) WHERE id = ?",
    )
    .bind(checkpoint_timeout_secs)
    .bind(event_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn update_bomb_penalty(
    pool: &Pool,
    event_id: i64,
    checkpoint_penalty_ms: i64,
) -> Result<(), sqlx::Error> {
    let _ = sqlx::query(
        "UPDATE events SET mode = json_set(mode, '$.checkpoint_penalty_ms', ?) WHERE id = ?",
    )
    .bind(checkpoint_penalty_ms)
    .bind(event_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn update_bomb_collision_penalty(
    pool: &Pool,
    event_id: i64,
    collision_max_penalty_ms: i64,
) -> Result<(), sqlx::Error> {
    let _ = sqlx::query(
        "UPDATE events SET mode = json_set(mode, '$.collision_max_penalty_ms', ?) WHERE id = ?",
    )
    .bind(collision_max_penalty_ms)
    .bind(event_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn update_vehicle_restrictions(
    pool: &Pool,
    event_id: i64,
    vehicles: &[Vehicle],
) -> Result<(), sqlx::Error> {
    let _ = sqlx::query("UPDATE events SET allowed_vehicles = ? WHERE id = ?")
        .bind(Json(vehicles.to_vec()))
        .bind(event_id)
        .execute(pool)
        .await?;
    Ok(())
}
