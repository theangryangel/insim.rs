use sqlx::Row;

use super::{ClimbTime, Pool};

pub async fn insert_climb_time(
    pool: &Pool,
    event_id: i64,
    uname: &str,
    vehicle: &str,
    time_ms: i64,
) -> Result<i64, sqlx::Error> {
    let row = sqlx::query(
        "INSERT INTO climb_times (event_id, user_id, vehicle, time_ms)
         VALUES (?, (SELECT id FROM users WHERE uname = ?), ?, ?)
         RETURNING id",
    )
    .bind(event_id)
    .bind(uname)
    .bind(vehicle)
    .bind(time_ms)
    .fetch_one(pool)
    .await?;
    Ok(row.get("id"))
}

pub async fn climb_best_times(
    pool: &Pool,
    event_id: i64,
) -> Result<Vec<ClimbTime>, sqlx::Error> {
    sqlx::query_as(
        "SELECT ct.id, ct.event_id, u.uname, u.pname, ct.vehicle, ct.time_ms, ct.set_at
         FROM climb_times ct
         JOIN users u ON u.id = ct.user_id
         INNER JOIN (
             SELECT user_id, MIN(time_ms) AS best
             FROM climb_times
             WHERE event_id = ?
             GROUP BY user_id
         ) pb ON ct.user_id = pb.user_id AND ct.time_ms = pb.best AND ct.event_id = ?
         ORDER BY ct.time_ms ASC",
    )
    .bind(event_id)
    .bind(event_id)
    .fetch_all(pool)
    .await
}

pub async fn climb_all_times(
    pool: &Pool,
    event_id: i64,
) -> Result<Vec<ClimbTime>, sqlx::Error> {
    sqlx::query_as(
        "SELECT ct.id, ct.event_id, u.uname, u.pname, ct.vehicle, ct.time_ms, ct.set_at
         FROM climb_times ct
         JOIN users u ON u.id = ct.user_id
         WHERE ct.event_id = ?
         ORDER BY ct.time_ms ASC",
    )
    .bind(event_id)
    .fetch_all(pool)
    .await
}

pub async fn climb_personal_best(
    pool: &Pool,
    event_id: i64,
    uname: &str,
) -> Result<Option<ClimbTime>, sqlx::Error> {
    sqlx::query_as(
        "SELECT ct.id, ct.event_id, u.uname, u.pname, ct.vehicle, ct.time_ms, ct.set_at
         FROM climb_times ct
         JOIN users u ON u.id = ct.user_id
         WHERE ct.event_id = ? AND u.uname = ?
         ORDER BY ct.time_ms ASC
         LIMIT 1",
    )
    .bind(event_id)
    .bind(uname)
    .fetch_optional(pool)
    .await
}
