use insim::core::track::Track;
use sqlx::{Row, types::Json};

use super::{Pool, Session, SessionMode};

pub struct CreateMetronomeParams {
    pub track: Track,
    pub layout: String,
    pub rounds: i64,
    pub target_ms: i64,
    pub max_scorers: i64,
    pub lobby_duration_secs: i64,
    pub name: Option<String>,
    pub description: Option<String>,
    pub scheduled_at: Option<String>,
}

pub struct CreateShortcutParams {
    pub track: Track,
    pub layout: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub scheduled_at: Option<String>,
}

pub struct UpdateSessionParams<'a> {
    pub track: Track,
    pub layout: &'a str,
    pub name: Option<&'a str>,
    pub description: Option<&'a str>,
    pub scheduled_at: Option<&'a str>,
    pub writeup: Option<&'a str>,
}

pub async fn all_sessions(pool: &Pool) -> Result<Vec<Session>, sqlx::Error> {
    sqlx::query_as(
        "SELECT id, mode, status, track, layout, created_at, started_at, ended_at, scheduled_at, name, description, writeup
         FROM sessions ORDER BY id DESC",
    )
    .fetch_all(pool)
    .await
}

pub async fn get_session(pool: &Pool, id: i64) -> Result<Option<Session>, sqlx::Error> {
    sqlx::query_as(
        "SELECT id, mode, status, track, layout, created_at, started_at, ended_at, scheduled_at, name, description, writeup
         FROM sessions WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn active_session(pool: &Pool) -> Result<Option<Session>, sqlx::Error> {
    sqlx::query_as(
        "SELECT id, mode, status, track, layout, created_at, started_at, ended_at, scheduled_at, name, description, writeup
         FROM sessions WHERE status = 'ACTIVE'
         LIMIT 1",
    )
    .fetch_optional(pool)
    .await
}

pub async fn pending_session(pool: &Pool, id: i64) -> Result<Option<Session>, sqlx::Error> {
    sqlx::query_as(
        "SELECT id, mode, status, track, layout, created_at, started_at, ended_at, scheduled_at, name, description, writeup
         FROM sessions WHERE id = ? AND status = 'PENDING'",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn upcoming_sessions(pool: &Pool) -> Result<Vec<Session>, sqlx::Error> {
    sqlx::query_as(
        "SELECT id, mode, status, track, layout, created_at, started_at, ended_at, scheduled_at, name, description, writeup
         FROM sessions WHERE status = 'PENDING' ORDER BY id DESC LIMIT 10",
    )
    .fetch_all(pool)
    .await
}

pub async fn next_scheduled_session(pool: &Pool) -> Result<Option<Session>, sqlx::Error> {
    sqlx::query_as(
        "SELECT id, mode, status, track, layout, created_at, started_at, ended_at, scheduled_at, name, description, writeup
         FROM sessions
         WHERE status = 'PENDING' AND scheduled_at IS NOT NULL AND scheduled_at <= datetime('now')
         ORDER BY scheduled_at ASC
         LIMIT 1",
    )
    .fetch_optional(pool)
    .await
}

pub async fn create_metronome_session(
    pool: &Pool,
    p: &CreateMetronomeParams,
) -> Result<i64, sqlx::Error> {
    let row = sqlx::query(
        "INSERT INTO sessions (mode, track, layout, name, description, scheduled_at) VALUES (?, ?, ?, ?, ?, ?) RETURNING id",
    )
    .bind(Json(SessionMode::Metronome { rounds: p.rounds, target_ms: p.target_ms, max_scorers: p.max_scorers, current_round: 0, lobby_duration_secs: p.lobby_duration_secs }))
    .bind(p.track.to_string())
    .bind(&p.layout)
    .bind(p.name.as_deref())
    .bind(p.description.as_deref())
    .bind(p.scheduled_at.as_deref())
    .fetch_one(pool)
    .await?;
    Ok(row.get("id"))
}

pub async fn create_shortcut_session(
    pool: &Pool,
    p: &CreateShortcutParams,
) -> Result<i64, sqlx::Error> {
    let row = sqlx::query(
        "INSERT INTO sessions (mode, track, layout, name, description, scheduled_at) VALUES (?, ?, ?, ?, ?, ?) RETURNING id",
    )
    .bind(Json(SessionMode::Shortcut))
    .bind(p.track.to_string())
    .bind(&p.layout)
    .bind(p.name.as_deref())
    .bind(p.description.as_deref())
    .bind(p.scheduled_at.as_deref())
    .fetch_one(pool)
    .await?;
    let id: i64 = row.get("id");

    let _ = sqlx::query("INSERT INTO shortcut_sessions (session_id) VALUES (?)")
        .bind(id)
        .execute(pool)
        .await?;

    Ok(id)
}

pub async fn switch_session(pool: &Pool, session_id: i64) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;
    let _ = sqlx::query(
        "UPDATE sessions SET status = 'COMPLETED', ended_at = datetime('now') WHERE status = 'ACTIVE' AND id != ?",
    )
    .bind(session_id)
    .execute(&mut *tx)
    .await?;

    let _ = sqlx::query(
        "UPDATE sessions SET status = 'ACTIVE', started_at = datetime('now') WHERE id = ? AND status = 'PENDING'",
    )
    .bind(session_id)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(())
}

pub async fn activate_session(pool: &Pool, session_id: i64) -> Result<(), sqlx::Error> {
    let _ = sqlx::query(
        "UPDATE sessions SET status = 'ACTIVE', started_at = datetime('now') WHERE id = ? AND status = 'PENDING'",
    )
    .bind(session_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn complete_session(pool: &Pool, session_id: i64) -> Result<(), sqlx::Error> {
    let _ = sqlx::query(
        "UPDATE sessions SET status = 'COMPLETED', ended_at = datetime('now') WHERE id = ?",
    )
    .bind(session_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn cancel_session(pool: &Pool, session_id: i64) -> Result<(), sqlx::Error> {
    let _ = sqlx::query(
        "UPDATE sessions SET status = 'CANCELLED', ended_at = datetime('now') WHERE id = ?",
    )
    .bind(session_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn update_session(
    pool: &Pool,
    id: i64,
    p: &UpdateSessionParams<'_>,
) -> Result<(), sqlx::Error> {
    let _ = sqlx::query(
        "UPDATE sessions SET track = ?, layout = ?, name = ?, description = ?, scheduled_at = ?, writeup = ? WHERE id = ?",
    )
    .bind(p.track.to_string())
    .bind(p.layout)
    .bind(p.name)
    .bind(p.description)
    .bind(p.scheduled_at)
    .bind(p.writeup)
    .bind(id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn update_metronome_settings(
    pool: &Pool,
    session_id: i64,
    rounds: i64,
    target_ms: i64,
    max_scorers: i64,
) -> Result<(), sqlx::Error> {
    let _ = sqlx::query(
        "UPDATE sessions SET mode = json_set(mode, '$.rounds', ?, '$.target_ms', ?, '$.max_scorers', ?) WHERE id = ?",
    )
    .bind(rounds)
    .bind(target_ms)
    .bind(max_scorers)
    .bind(session_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn update_session_writeup(
    pool: &Pool,
    session_id: i64,
    writeup: &str,
) -> Result<(), sqlx::Error> {
    let _ = sqlx::query("UPDATE sessions SET writeup = ? WHERE id = ?")
        .bind(writeup)
        .bind(session_id)
        .execute(pool)
        .await?;
    Ok(())
}
