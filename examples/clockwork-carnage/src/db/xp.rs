use sqlx::FromRow;

use super::{Pool, Timestamp};

#[derive(Debug, Clone, FromRow)]
pub struct XpLeaderboardRow {
    pub uname: String,
    pub pname: String,
    pub total_xp: i64,
    pub events_played: i64,
    pub twitch_username: Option<String>,
    pub youtube_username: Option<String>,
}

pub async fn get_top_xp(pool: &Pool, limit: i64) -> Result<Vec<XpLeaderboardRow>, sqlx::Error> {
    sqlx::query_as(
        "SELECT u.uname, u.pname, u.twitch_username, u.youtube_username,
                SUM(x.amount) AS total_xp, COUNT(DISTINCT x.event_id) AS events_played
         FROM users_xp x
         JOIN users u ON u.id = x.user_id
         GROUP BY x.user_id
         ORDER BY total_xp DESC
         LIMIT ?",
    )
    .bind(limit)
    .fetch_all(pool)
    .await
}

pub async fn award_xp(
    pool: &Pool,
    uname: &str,
    amount: i64,
    reason: &str,
    event_id: Option<i64>,
) -> Result<(), sqlx::Error> {
    let now = Timestamp::now();
    sqlx::query(
        "INSERT INTO users_xp (user_id, event_id, amount, reason, recorded_at)
         VALUES ((SELECT id FROM users WHERE uname = ?), ?, ?, ?, ?)",
    )
    .bind(uname)
    .bind(event_id)
    .bind(amount)
    .bind(reason)
    .bind(now)
    .execute(pool)
    .await
    .map(|_| ())
}
