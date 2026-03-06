use sqlx::Row;

use super::{Pool, ShortcutTime};

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
         ORDER BY ct.time_ms ASC",
    )
    .bind(session_id)
    .bind(session_id)
    .fetch_all(pool)
    .await
}

pub async fn shortcut_all_times(
    pool: &Pool,
    session_id: i64,
) -> Result<Vec<ShortcutTime>, sqlx::Error> {
    sqlx::query_as(
        "SELECT ct.id, ct.session_id, u.uname, u.pname, ct.vehicle, ct.time_ms, ct.set_at
         FROM shortcut_times ct
         JOIN users u ON u.id = ct.user_id
         WHERE ct.session_id = ?
         ORDER BY ct.time_ms ASC",
    )
    .bind(session_id)
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
