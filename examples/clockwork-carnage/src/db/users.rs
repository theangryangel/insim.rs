use kitcar::presence::{Presence, PresenceEvent};
use sqlx::Row;
use tokio::task::JoinHandle;

use super::{Pool, User};

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
