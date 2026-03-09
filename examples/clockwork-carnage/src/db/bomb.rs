use super::{BombRun, Pool};

pub async fn insert_bomb_run(
    pool: &Pool,
    session_id: i64,
    uname: &str,
    vehicle: &str,
    checkpoint_count: i64,
    survival_ms: i64,
) -> Result<(), sqlx::Error> {
    let _ = sqlx::query(
        "INSERT INTO bomb_runs (session_id, user_id, vehicle, checkpoint_count, survival_ms)
         VALUES (?, (SELECT id FROM users WHERE uname = ?), ?, ?, ?)",
    )
    .bind(session_id)
    .bind(uname)
    .bind(vehicle)
    .bind(checkpoint_count)
    .bind(survival_ms)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn bomb_best_runs(pool: &Pool, session_id: i64) -> Result<Vec<BombRun>, sqlx::Error> {
    sqlx::query_as(
        "SELECT br.id, br.session_id, br.user_id, u.uname, u.pname, br.vehicle,
                br.checkpoint_count, br.survival_ms, br.recorded_at
         FROM bomb_runs br
         JOIN users u ON u.id = br.user_id
         INNER JOIN (
             SELECT user_id, MAX(checkpoint_count) AS best_cps
             FROM bomb_runs
             WHERE session_id = ?
             GROUP BY user_id
         ) pb ON br.user_id = pb.user_id AND br.checkpoint_count = pb.best_cps AND br.session_id = ?
         INNER JOIN (
             SELECT user_id, MAX(checkpoint_count) AS best_cps, MAX(survival_ms) AS best_survival
             FROM bomb_runs
             WHERE session_id = ?
             GROUP BY user_id
         ) pb2 ON br.user_id = pb2.user_id AND br.checkpoint_count = pb2.best_cps AND br.survival_ms = pb2.best_survival AND br.session_id = ?
         ORDER BY br.checkpoint_count DESC, br.survival_ms DESC",
    )
    .bind(session_id)
    .bind(session_id)
    .bind(session_id)
    .bind(session_id)
    .fetch_all(pool)
    .await
}

pub async fn bomb_all_runs(pool: &Pool, session_id: i64) -> Result<Vec<BombRun>, sqlx::Error> {
    sqlx::query_as(
        "SELECT br.id, br.session_id, br.user_id, u.uname, u.pname, br.vehicle,
                br.checkpoint_count, br.survival_ms, br.recorded_at
         FROM bomb_runs br
         JOIN users u ON u.id = br.user_id
         WHERE br.session_id = ?
         ORDER BY br.checkpoint_count DESC, br.survival_ms DESC",
    )
    .bind(session_id)
    .fetch_all(pool)
    .await
}
