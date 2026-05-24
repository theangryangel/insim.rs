use sqlx::Row;

use super::{MetronomeStanding, Pool};

pub async fn insert_metronome_lap(
    pool: &Pool,
    event_id: i64,
    uname: &str,
    delta_ms: i64,
) -> Result<i64, sqlx::Error> {
    let row = sqlx::query(
        "INSERT INTO metronome_results (event_id, user_id, delta_ms, recorded_at)
         VALUES ($1, (SELECT id FROM users WHERE uname = $2), $3, NOW())
         RETURNING id",
    )
    .bind(event_id)
    .bind(uname)
    .bind(delta_ms)
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
        SELECT u.uname, u.pname, u.twitch_username, u.youtube_username, MIN(r.delta_ms) AS best_delta_ms
        FROM metronome_results r
        JOIN users u ON u.id = r.user_id
        WHERE r.event_id = $1
        GROUP BY r.user_id, u.uname, u.pname, u.twitch_username, u.youtube_username
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
            twitch_username: row.get("twitch_username"),
            youtube_username: row.get("youtube_username"),
        })
        .collect())
}
