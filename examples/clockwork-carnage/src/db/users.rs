use kitcar::presence::{Presence, PresenceEvent};
use sqlx::Row;
use tokio::task::JoinHandle;

use super::{Pool, Timestamp, User};

pub async fn get_user_by_id(pool: &Pool, id: i64) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as(
        "SELECT id, uname, pname, last_seen, oauth_access_token, admin, twitch_username, youtube_username FROM users WHERE id = ?",
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
    let now = Timestamp::now();
    sqlx::query_as(
        "INSERT INTO users (uname, pname, last_seen, oauth_access_token)
         VALUES (?, ?, ?, ?)
         ON CONFLICT(uname) DO UPDATE SET
           pname = excluded.pname,
           oauth_access_token = excluded.oauth_access_token,
           last_seen = excluded.last_seen
         RETURNING id, uname, pname, last_seen, oauth_access_token, admin, twitch_username, youtube_username",
    )
    .bind(uname)
    .bind(pname)
    .bind(now)
    .bind(token)
    .fetch_one(pool)
    .await
}

pub async fn upsert_user(pool: &Pool, uname: &str, pname: &str) -> Result<i64, sqlx::Error> {
    let now = Timestamp::now();
    let row = sqlx::query(
        "INSERT INTO users (uname, pname, last_seen) VALUES (?, ?, ?)
         ON CONFLICT(uname) DO UPDATE SET pname = excluded.pname, last_seen = excluded.last_seen
         RETURNING id",
    )
    .bind(uname)
    .bind(pname)
    .bind(now)
    .fetch_one(pool)
    .await?;
    Ok(row.get("id"))
}

pub async fn update_user_profile(
    pool: &Pool,
    uname: &str,
    twitch_username: Option<&str>,
    youtube_username: Option<&str>,
) -> Result<(), sqlx::Error> {
    let _ = sqlx::query(
        "UPDATE users SET twitch_username = ?, youtube_username = ? WHERE uname = ?",
    )
    .bind(twitch_username)
    .bind(youtube_username)
    .bind(uname)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn users_for_unames(pool: &Pool, unames: &[String]) -> Result<Vec<User>, sqlx::Error> {
    if unames.is_empty() {
        return Ok(vec![]);
    }
    let placeholders = unames.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
    let sql = format!(
        "SELECT id, uname, pname, last_seen, oauth_access_token, admin, twitch_username, youtube_username FROM users WHERE uname IN ({placeholders})"
    );
    let mut query = sqlx::query_as(&sql);
    for uname in unames {
        query = query.bind(uname);
    }
    query.fetch_all(pool).await
}

#[derive(Debug, thiserror::Error)]
pub enum UserSyncError {
    #[error("presence event channel closed")]
    PresenceClosed,
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
