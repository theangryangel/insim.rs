use sqlx::Row;

use super::{Pool, MetronomeResult, MetronomeStanding};

pub async fn update_metronome_round(
    pool: &Pool,
    event_id: i64,
    round: i64,
) -> Result<(), sqlx::Error> {
    let _ = sqlx::query(
        r#"
        UPDATE events
        SET mode = json_set(mode, '$.current_round', ?)
        WHERE id = ?
        "#,
    )
    .bind(round)
    .bind(event_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn insert_metronome_result(
    pool: &Pool,
    event_id: i64,
    round: i64,
    uname: &str,
    delta_ms: i64,
    points: i64,
) -> Result<i64, sqlx::Error> {
    let row = sqlx::query(
        "INSERT INTO metronome_results (event_id, round, user_id, delta_ms, points)
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

pub async fn metronome_standings(
    pool: &Pool,
    event_id: i64,
) -> Result<Vec<MetronomeStanding>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT u.uname, u.pname, SUM(r.points) AS total_points
        FROM metronome_results r
        JOIN users u ON u.id = r.user_id
        JOIN events s ON s.id = r.event_id
        WHERE r.event_id = ?
          AND s.mode ->> '$.type' = 'metronome' -- Guard rail
        GROUP BY r.user_id
        ORDER BY total_points DESC
        "#,
    )
    .bind(event_id)
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(|row| MetronomeStanding {
        uname: row.get("uname"),
        pname: row.get("pname"),
        total_points: row.get("total_points"),
    }).collect())
}

pub async fn metronome_all_results(
    pool: &Pool,
    event_id: i64,
) -> Result<Vec<MetronomeResult>, sqlx::Error> {
    sqlx::query_as(
        "SELECT r.id, r.event_id, r.round, u.uname, u.pname, r.delta_ms, r.points, r.recorded_at
         FROM metronome_results r
         JOIN users u ON u.id = r.user_id
         WHERE r.event_id = ?
         ORDER BY r.round ASC, r.points DESC",
    )
    .bind(event_id)
    .fetch_all(pool)
    .await
}
