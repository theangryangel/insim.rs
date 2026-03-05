//! SQLite database layer for Clockwork Carnage.

use std::{collections::HashMap, fmt, str::FromStr};

use insim::core::track::Track;
use kitcar::presence::{Presence, PresenceEvent};
use sqlx::{
    FromRow, Row,
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions}, types::Json,
};
use tokio::task::JoinHandle;

pub type Pool = sqlx::SqlitePool;

pub async fn connect(path: &str) -> Result<Pool, sqlx::Error> {
    let options = SqliteConnectOptions::from_str(path)?
        .journal_mode(SqliteJournalMode::Wal)
        .create_if_missing(true);
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    Ok(pool)
}

// -- Enums --------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SessionMode {
    Metronome {
        rounds: i64,
        target_ms: i64,
        max_scorers: i64,
        #[serde(default)]
        current_round: i64,
    },
    Shortcut,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionStatus {
    Pending,
    Active,
    Completed,
    Cancelled,
}

impl fmt::Display for SessionStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Pending => f.write_str("PENDING"),
            Self::Active => f.write_str("ACTIVE"),
            Self::Completed => f.write_str("COMPLETED"),
            Self::Cancelled => f.write_str("CANCELLED"),
        }
    }
}

impl TryFrom<String> for SessionStatus {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.as_str() {
            "PENDING" => Ok(Self::Pending),
            "ACTIVE" => Ok(Self::Active),
            "COMPLETED" => Ok(Self::Completed),
            "CANCELLED" => Ok(Self::Cancelled),
            other => Err(format!("unknown session status: {other}")),
        }
    }
}

// -- Row types ----------------------------------------------------------------

#[derive(Clone, FromRow)]
pub struct User {
    pub id: i64,
    pub uname: String,
    pub pname: String,
    pub last_seen: String,
    pub oauth_access_token: Option<String>,
    pub admin: bool,
}

impl std::fmt::Debug for User {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("User")
            .field("id", &self.id)
            .field("uname", &self.uname)
            .field("pname", &self.pname)
            .field("last_seen", &self.last_seen)
            .field("oauth_access_token", &"[redacted]")
            .finish()
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct Session {
    pub id: i64,
    pub mode: Json<SessionMode>,
    #[sqlx(try_from = "String")]
    pub status: SessionStatus,
    #[sqlx(try_from = "String")]
    pub track: Track,
    pub layout: String,
    pub created_at: String,
    pub started_at: Option<String>,
    pub ended_at: Option<String>,
    pub scheduled_at: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub writeup: Option<String>,
}

#[derive(Debug, Clone)]
pub struct MetronomeStanding {
    pub uname: String,
    pub pname: String,
    pub total_points: i64,
}

#[derive(Debug, Clone, FromRow)]
pub struct MetronomeResult {
    pub id: i64,
    pub session_id: i64,
    pub round: i64,
    pub uname: String,
    pub pname: String,
    pub delta_ms: i64,
    pub points: i64,
    pub recorded_at: String,
}

#[derive(Debug, Clone, FromRow)]
pub struct ShortcutTime {
    pub id: i64,
    pub session_id: i64,
    pub uname: String,
    pub pname: String,
    pub vehicle: String,
    pub time_ms: i64,
    pub set_at: String,
}

// -- List / get queries -------------------------------------------------------

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


// -- User queries -------------------------------------------------------------

pub async fn get_user_by_id(pool: &Pool, id: i64) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as(
        "SELECT id, uname, pname, last_seen, oauth_access_token, admin FROM users WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn upsert_user_with_token(
    pool: &Pool,
    uname: &str,
    pname: &str,
    token: &str,
) -> Result<User, sqlx::Error> {
    sqlx::query_as(
        "INSERT INTO users (uname, pname, oauth_access_token)
         VALUES (?, ?, ?)
         ON CONFLICT(uname) DO UPDATE SET
           pname = excluded.pname,
           oauth_access_token = excluded.oauth_access_token,
           last_seen = datetime('now')
         RETURNING id, uname, pname, last_seen, oauth_access_token, admin",
    )
    .bind(uname)
    .bind(pname)
    .bind(token)
    .fetch_one(pool)
    .await
}

pub async fn upsert_user(pool: &Pool, uname: &str, pname: &str) -> Result<i64, sqlx::Error> {
    let row = sqlx::query(
        "INSERT INTO users (uname, pname) VALUES (?, ?)
         ON CONFLICT(uname) DO UPDATE SET pname = excluded.pname, last_seen = datetime('now')
         RETURNING id",
    )
    .bind(uname)
    .bind(pname)
    .fetch_one(pool)
    .await?;
    Ok(row.get("id"))
}

/// Background task that subscribes to presence events and upserts users.
pub fn spawn_user_sync(presence: &Presence, pool: Pool) -> JoinHandle<Result<(), UserSyncError>> {
    let mut events = presence.subscribe_events();

    tokio::spawn(async move {
        loop {
            match events.recv().await {
                Ok(PresenceEvent::Connected(conn)) => {
                    if let Err(e) = upsert_user(&pool, &conn.uname, &conn.pname).await {
                        tracing::warn!("Failed to upsert user {}: {e}", conn.uname);
                    }
                },
                Ok(PresenceEvent::Renamed {
                    uname, new_pname, ..
                }) => {
                    if let Err(e) = upsert_user(&pool, &uname, &new_pname).await {
                        tracing::warn!("Failed to upsert user {uname}: {e}");
                    }
                },
                Ok(_) => {},
                Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                    tracing::warn!("User sync lagged by {n} events");
                },
                Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                    return Err(UserSyncError::PresenceClosed);
                },
            }
        }
    })
}

#[derive(Debug, thiserror::Error)]
pub enum UserSyncError {
    #[error("presence event channel closed")]
    PresenceClosed,
}

// -- Session creation ---------------------------------------------------------

pub async fn create_metronome_session(
    pool: &Pool,
    track: &Track,
    layout: &str,
    rounds: i64,
    target_ms: i64,
    max_scorers: i64,
    name: Option<&str>,
    description: Option<&str>,
    scheduled_at: Option<&str>,
) -> Result<i64, sqlx::Error> {
    let row = sqlx::query(
        "INSERT INTO sessions (mode, track, layout, name, description, scheduled_at) VALUES (?, ?, ?, ?, ?, ?) RETURNING id",
    )
    .bind(Json(SessionMode::Metronome { rounds, target_ms, max_scorers, current_round: 0 }))
    .bind(track.to_string())
    .bind(layout)
    .bind(name)
    .bind(description)
    .bind(scheduled_at)
    .fetch_one(pool)
    .await?;
    let id: i64 = row.get("id");

    Ok(id)
}

pub async fn create_shortcut_session(
    pool: &Pool,
    track: &Track,
    layout: &str,
    name: Option<&str>,
    description: Option<&str>,
    scheduled_at: Option<&str>,
) -> Result<i64, sqlx::Error> {
    let row = sqlx::query(
        "INSERT INTO sessions (mode, track, layout, name, description, scheduled_at) VALUES (?, ?, ?, ?, ?, ?) RETURNING id",
    )
    .bind(Json(SessionMode::Shortcut))
    .bind(track.to_string())
    .bind(layout)
    .bind(name)
    .bind(description)
    .bind(scheduled_at)
    .fetch_one(pool)
    .await?;
    let id: i64 = row.get("id");

    let _ = sqlx::query("INSERT INTO shortcut_sessions (session_id) VALUES (?)")
        .bind(id)
        .execute(pool)
        .await?;

    Ok(id)
}

// -- Session lifecycle --------------------------------------------------------

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

// -- Metronome queries --------------------------------------------------------

pub async fn update_metronome_round(
    pool: &Pool,
    session_id: i64,
    round: i64,
) -> Result<(), sqlx::Error> {
    let _ = sqlx::query(
        r#"
        UPDATE sessions 
        SET mode = json_set(mode, '$.current_round', ?) 
        WHERE id = ?
        "#,
    )
    .bind(round)
    .bind(session_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn insert_metronome_result(
    pool: &Pool,
    session_id: i64,
    round: i64,
    uname: &str,
    delta_ms: i64,
    points: i64,
) -> Result<i64, sqlx::Error> {
    let row = sqlx::query(
        "INSERT INTO metronome_results (session_id, round, user_id, delta_ms, points)
         VALUES (?, ?, (SELECT id FROM users WHERE uname = ?), ?, ?)
         RETURNING id",
    )
    .bind(session_id)
    .bind(round)
    .bind(uname)
    .bind(delta_ms)
    .bind(points)
    .fetch_one(pool)
    .await?;
    Ok(row.get("id"))
}

pub async fn metronome_standings(
    pool: &Pool,
    session_id: i64,
) -> Result<Vec<MetronomeStanding>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT u.uname, u.pname, SUM(r.points) AS total_points
        FROM metronome_results r
        JOIN users u ON u.id = r.user_id
        JOIN sessions s ON s.id = r.session_id
        WHERE r.session_id = ? 
          AND s.mode ->> '$.type' = 'METRONOME' -- Guard rail
        GROUP BY r.user_id
        ORDER BY total_points DESC
        "#,
    )
    .bind(session_id)
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(|row| MetronomeStanding {
        uname: row.get("uname"),
        pname: row.get("pname"),
        total_points: row.get("total_points"),
    }).collect())
}

pub async fn metronome_round_results(
    pool: &Pool,
    session_id: i64,
    round: i64,
) -> Result<Vec<MetronomeResult>, sqlx::Error> {
    sqlx::query_as(
        "SELECT r.id, r.session_id, r.round, u.uname, u.pname, r.delta_ms, r.points, r.recorded_at
         FROM metronome_results r
         JOIN users u ON u.id = r.user_id
         WHERE r.session_id = ? AND r.round = ?
         ORDER BY r.points DESC",
    )
    .bind(session_id)
    .bind(round)
    .fetch_all(pool)
    .await
}

// -- Shortcut queries ---------------------------------------------------------

pub async fn insert_shortcut_time(
    pool: &Pool,
    session_id: i64,
    uname: &str,
    vehicle: &str,
    time_ms: i64,
) -> Result<i64, sqlx::Error> {
    let row = sqlx::query(
        "INSERT INTO shortcut_times (session_id, user_id, vehicle, time_ms)
         VALUES (?, (SELECT id FROM users WHERE uname = ?), ?, ?)
         RETURNING id",
    )
    .bind(session_id)
    .bind(uname)
    .bind(vehicle)
    .bind(time_ms)
    .fetch_one(pool)
    .await?;
    Ok(row.get("id"))
}

pub async fn shortcut_best_times(
    pool: &Pool,
    session_id: i64,
    limit: i64,
) -> Result<Vec<ShortcutTime>, sqlx::Error> {
    sqlx::query_as(
        "SELECT ct.id, ct.session_id, u.uname, u.pname, ct.vehicle, ct.time_ms, ct.set_at
         FROM shortcut_times ct
         JOIN users u ON u.id = ct.user_id
         INNER JOIN (
             SELECT user_id, MIN(time_ms) AS best
             FROM shortcut_times
             WHERE session_id = ?
             GROUP BY user_id
         ) pb ON ct.user_id = pb.user_id AND ct.time_ms = pb.best AND ct.session_id = ?
         ORDER BY ct.time_ms ASC
         LIMIT ?",
    )
    .bind(session_id)
    .bind(session_id)
    .bind(limit)
    .fetch_all(pool)
    .await
}

pub async fn shortcut_personal_best(
    pool: &Pool,
    session_id: i64,
    uname: &str,
) -> Result<Option<ShortcutTime>, sqlx::Error> {
    sqlx::query_as(
        "SELECT ct.id, ct.session_id, u.uname, u.pname, ct.vehicle, ct.time_ms, ct.set_at
         FROM shortcut_times ct
         JOIN users u ON u.id = ct.user_id
         WHERE ct.session_id = ? AND u.uname = ?
         ORDER BY ct.time_ms ASC
         LIMIT 1",
    )
    .bind(session_id)
    .bind(uname)
    .fetch_optional(pool)
    .await
}

// -- Session metadata updates -------------------------------------------------

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

pub async fn update_session_details(
    pool: &Pool,
    session_id: i64,
    name: Option<&str>,
    description: Option<&str>,
) -> Result<(), sqlx::Error> {
    let _ = sqlx::query("UPDATE sessions SET name = ?, description = ? WHERE id = ?")
        .bind(name)
        .bind(description)
        .bind(session_id)
        .execute(pool)
        .await?;
    Ok(())
}
