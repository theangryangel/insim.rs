//! SQLite database layer for Clockwork Carnage.

use std::str::FromStr;

use kitcar::presence::{Presence, PresenceEvent};
use sqlx::{
    FromRow, Row,
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions},
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

// -- Row types ----------------------------------------------------------------

#[derive(Debug, Clone, FromRow)]
pub struct User {
    pub id: i64,
    pub uname: String,
    pub pname: String,
    pub last_seen: String,
}

#[derive(Debug, Clone, FromRow)]
pub struct Challenge {
    pub id: i64,
    pub track: String,
    pub layout: String,
    pub started_at: String,
    pub ended_at: Option<String>,
}

#[derive(Debug, Clone, FromRow)]
pub struct ChallengeTime {
    pub id: i64,
    pub challenge_id: i64,
    pub uname: String,
    pub pname: String,
    pub vehicle: String,
    pub time_ms: i64,
    pub set_at: String,
}

#[derive(Debug, Clone, FromRow)]
pub struct Event {
    pub id: i64,
    pub track: String,
    pub layout: String,
    pub rounds: i64,
    pub target_ms: i64,
    pub current_round: i64,
    pub started_at: String,
    pub ended_at: Option<String>,
}

#[derive(Debug, Clone)]
pub struct EventStanding {
    pub uname: String,
    pub pname: String,
    pub total_points: i64,
}

#[derive(Debug, Clone, FromRow)]
pub struct EventRoundResult {
    pub id: i64,
    pub event_id: i64,
    pub round: i64,
    pub uname: String,
    pub pname: String,
    pub delta_ms: i64,
    pub points: i64,
    pub recorded_at: String,
}

// -- User queries -------------------------------------------------------------

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

// -- Challenge queries --------------------------------------------------------

pub async fn create_challenge(pool: &Pool, track: &str, layout: &str) -> Result<i64, sqlx::Error> {
    let row = sqlx::query("INSERT INTO challenges (track, layout) VALUES (?, ?) RETURNING id")
        .bind(track)
        .bind(layout)
        .fetch_one(pool)
        .await?;
    Ok(row.get("id"))
}

pub async fn end_challenge(pool: &Pool, challenge_id: i64) -> Result<(), sqlx::Error> {
    let _ = sqlx::query("UPDATE challenges SET ended_at = datetime('now') WHERE id = ?")
        .bind(challenge_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn active_challenge(
    pool: &Pool,
    track: &str,
    layout: &str,
) -> Result<Option<Challenge>, sqlx::Error> {
    sqlx::query_as(
        "SELECT id, track, layout, started_at, ended_at
         FROM challenges
         WHERE track = ? AND layout = ? AND ended_at IS NULL
         ORDER BY id DESC LIMIT 1",
    )
    .bind(track)
    .bind(layout)
    .fetch_optional(pool)
    .await
}

pub async fn insert_challenge_time(
    pool: &Pool,
    challenge_id: i64,
    uname: &str,
    vehicle: &str,
    time_ms: i64,
) -> Result<i64, sqlx::Error> {
    let row = sqlx::query(
        "INSERT INTO challenge_times (challenge_id, user_id, vehicle, time_ms)
         VALUES (?, (SELECT id FROM users WHERE uname = ?), ?, ?)
         RETURNING id",
    )
    .bind(challenge_id)
    .bind(uname)
    .bind(vehicle)
    .bind(time_ms)
    .fetch_one(pool)
    .await?;
    Ok(row.get("id"))
}

pub async fn challenge_best_times(
    pool: &Pool,
    challenge_id: i64,
    limit: i64,
) -> Result<Vec<ChallengeTime>, sqlx::Error> {
    sqlx::query_as(
        "SELECT ct.id, ct.challenge_id, u.uname, u.pname, ct.vehicle, ct.time_ms, ct.set_at
         FROM challenge_times ct
         JOIN users u ON u.id = ct.user_id
         INNER JOIN (
             SELECT user_id, MIN(time_ms) AS best
             FROM challenge_times
             WHERE challenge_id = ?
             GROUP BY user_id
         ) pb ON ct.user_id = pb.user_id AND ct.time_ms = pb.best AND ct.challenge_id = ?
         ORDER BY ct.time_ms ASC
         LIMIT ?",
    )
    .bind(challenge_id)
    .bind(challenge_id)
    .bind(limit)
    .fetch_all(pool)
    .await
}

pub async fn challenge_personal_best(
    pool: &Pool,
    challenge_id: i64,
    uname: &str,
) -> Result<Option<ChallengeTime>, sqlx::Error> {
    sqlx::query_as(
        "SELECT ct.id, ct.challenge_id, u.uname, u.pname, ct.vehicle, ct.time_ms, ct.set_at
         FROM challenge_times ct
         JOIN users u ON u.id = ct.user_id
         WHERE ct.challenge_id = ? AND u.uname = ?
         ORDER BY ct.time_ms ASC
         LIMIT 1",
    )
    .bind(challenge_id)
    .bind(uname)
    .fetch_optional(pool)
    .await
}

// -- Event queries ------------------------------------------------------------

pub async fn create_event(
    pool: &Pool,
    track: &str,
    layout: &str,
    rounds: i64,
    target_ms: i64,
) -> Result<i64, sqlx::Error> {
    let row = sqlx::query(
        "INSERT INTO events (track, layout, rounds, target_ms) VALUES (?, ?, ?, ?) RETURNING id",
    )
    .bind(track)
    .bind(layout)
    .bind(rounds)
    .bind(target_ms)
    .fetch_one(pool)
    .await?;
    Ok(row.get("id"))
}

pub async fn end_event(pool: &Pool, event_id: i64) -> Result<(), sqlx::Error> {
    let _ = sqlx::query("UPDATE events SET ended_at = datetime('now') WHERE id = ?")
        .bind(event_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn active_event(
    pool: &Pool,
    track: &str,
    layout: &str,
) -> Result<Option<Event>, sqlx::Error> {
    sqlx::query_as(
        "SELECT id, track, layout, rounds, target_ms, current_round, started_at, ended_at
         FROM events
         WHERE track = ? AND layout = ? AND ended_at IS NULL
         ORDER BY id DESC LIMIT 1",
    )
    .bind(track)
    .bind(layout)
    .fetch_optional(pool)
    .await
}

pub async fn update_event_round(pool: &Pool, event_id: i64, round: i64) -> Result<(), sqlx::Error> {
    let _ = sqlx::query("UPDATE events SET current_round = ? WHERE id = ?")
        .bind(round)
        .bind(event_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn insert_event_round_result(
    pool: &Pool,
    event_id: i64,
    round: i64,
    uname: &str,
    delta_ms: i64,
    points: i64,
) -> Result<i64, sqlx::Error> {
    let row = sqlx::query(
        "INSERT INTO event_round_results (event_id, round, user_id, delta_ms, points)
         VALUES (?, ?, (SELECT id FROM users WHERE uname = ?), ?, ?)
         RETURNING id",
    )
    .bind(event_id)
    .bind(round)
    .bind(uname)
    .bind(delta_ms)
    .bind(points)
    .fetch_one(pool)
    .await?;
    Ok(row.get("id"))
}

pub async fn event_standings(
    pool: &Pool,
    event_id: i64,
) -> Result<Vec<EventStanding>, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT u.uname, u.pname, SUM(r.points) AS total_points
         FROM event_round_results r
         JOIN users u ON u.id = r.user_id
         WHERE r.event_id = ?
         GROUP BY r.user_id
         ORDER BY total_points DESC",
    )
    .bind(event_id)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| EventStanding {
            uname: row.get("uname"),
            pname: row.get("pname"),
            total_points: row.get("total_points"),
        })
        .collect())
}

pub async fn event_round_results(
    pool: &Pool,
    event_id: i64,
    round: i64,
) -> Result<Vec<EventRoundResult>, sqlx::Error> {
    sqlx::query_as(
        "SELECT r.id, r.event_id, r.round, u.uname, u.pname, r.delta_ms, r.points, r.recorded_at
         FROM event_round_results r
         JOIN users u ON u.id = r.user_id
         WHERE r.event_id = ? AND r.round = ?
         ORDER BY r.points DESC",
    )
    .bind(event_id)
    .bind(round)
    .fetch_all(pool)
    .await
}
