use sqlx::Row;

use super::{MetronomeStanding, Pool, Timestamp};

pub async fn insert_metronome_lap(
    pool: &Pool,
    event_id: i64,
    uname: &str,
    delta_ms: i64,
) -> Result<i64, sqlx::Error> {
    let now = Timestamp::now();
    let row = sqlx::query(
        "INSERT INTO metronome_results (event_id, user_id, delta_ms, recorded_at)
         VALUES (?, (SELECT id FROM users WHERE uname = ?), ?, ?)
         RETURNING id",
    )
    .bind(event_id)
    .bind(uname)
    .bind(delta_ms)
    .bind(now)
    .fetch_one(pool)
    .await?;
    Ok(row.get("id"))
}

pub async fn metronome_standings(
    pool: &Pool,
    event_id: i64,
) -> Result<Vec<MetronomeStanding>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT u.uname, u.pname, MIN(r.delta_ms) AS best_delta_ms
        FROM metronome_results r
        JOIN users u ON u.id = r.user_id
        WHERE r.event_id = ?
        GROUP BY r.user_id
        ORDER BY best_delta_ms ASC
        "#,
    )
    .bind(event_id)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| MetronomeStanding {
            uname: row.get("uname"),
            pname: row.get("pname"),
            best_delta_ms: row.get("best_delta_ms"),
        })
        .collect())
}

pub async fn metronome_personal_best(
    pool: &Pool,
    event_id: i64,
    uname: &str,
) -> Result<Option<i64>, sqlx::Error> {
    sqlx::query_scalar::<_, Option<i64>>(
        "SELECT MIN(r.delta_ms) FROM metronome_results r
         JOIN users u ON u.id = r.user_id
         WHERE r.event_id = ? AND u.uname = ?",
    )
    .bind(event_id)
    .bind(uname)
    .fetch_one(pool)
    .await
}
