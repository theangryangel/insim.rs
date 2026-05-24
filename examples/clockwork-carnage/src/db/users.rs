use super::{Pool, User};

pub async fn upsert_user_with_token(
    pool: &Pool,
    uname: &str,
    pname: &str,
    token: &str,
) -> Result<User, sqlx::Error> {
    sqlx::query_as(
        "INSERT INTO users (uname, pname, last_seen, oauth_access_token)
         VALUES ($1, $2, NOW(), $3)
         ON CONFLICT(uname) DO UPDATE SET
           pname = EXCLUDED.pname,
           oauth_access_token = EXCLUDED.oauth_access_token,
           last_seen = EXCLUDED.last_seen
         RETURNING id, uname, pname, last_seen, oauth_access_token, admin, twitch_username, youtube_username",
    )
    .bind(uname)
    .bind(pname)
    .bind(token)
    .fetch_one(pool)
    .await
}

pub async fn get_user_by_id(pool: &Pool, id: i64) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as(
        "SELECT id, uname, pname, last_seen, oauth_access_token, admin, twitch_username, youtube_username
         FROM users WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn update_user_profile(
    pool: &Pool,
    uname: &str,
    twitch_username: Option<&str>,
    youtube_username: Option<&str>,
) -> Result<(), sqlx::Error> {
    let _ = sqlx::query(
        "UPDATE users SET twitch_username = $1, youtube_username = $2 WHERE uname = $3",
    )
    .bind(twitch_username)
    .bind(youtube_username)
    .bind(uname)
    .execute(pool)
    .await?;
    Ok(())
}
