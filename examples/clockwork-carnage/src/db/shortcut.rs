use super::{Pool, ShortcutTime};

pub async fn insert_shortcut_time(
    pool: &Pool,
    event_id: i64,
    uname: &str,
    vehicle: &str,
    time_ms: i64,
) -> Result<i64, sqlx::Error> {
    use sqlx::Row;
    let row = sqlx::query(
        "INSERT INTO shortcut_times (event_id, user_id, vehicle, time_ms, set_at)
         VALUES ($1, (SELECT id FROM users WHERE uname = $2), $3, $4, NOW())
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

pub async fn shortcut_best_times(
    pool: &Pool,
    event_id: i64,
) -> Result<Vec<ShortcutTime>, sqlx::Error> {
    sqlx::query_as(
        "SELECT u.uname, u.pname, u.twitch_username, u.youtube_username, ct.vehicle, ct.time_ms, ct.set_at
         FROM shortcut_times ct
         JOIN users u ON u.id = ct.user_id
         INNER JOIN (
             SELECT user_id, MIN(time_ms) AS best
             FROM shortcut_times
             WHERE event_id = $1
             GROUP BY user_id
         ) pb ON ct.user_id = pb.user_id AND ct.time_ms = pb.best AND ct.event_id = $2
         ORDER BY ct.time_ms ASC",
    )
    .bind(event_id)
    .bind(event_id)
    .fetch_all(pool)
    .await
}

pub async fn shortcut_all_times(
    pool: &Pool,
    event_id: i64,
) -> Result<Vec<ShortcutTime>, sqlx::Error> {
    sqlx::query_as(
        "SELECT u.uname, u.pname, u.twitch_username, u.youtube_username, ct.vehicle, ct.time_ms, ct.set_at
         FROM shortcut_times ct
         JOIN users u ON u.id = ct.user_id
         WHERE ct.event_id = $1
         ORDER BY ct.time_ms ASC",
    )
    .bind(event_id)
    .fetch_all(pool)
    .await
}
