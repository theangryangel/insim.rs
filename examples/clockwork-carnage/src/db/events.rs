use sqlx::Row;

use super::{Era, Event, EventMode, EventStatus, Pool};

const EVENT_COLUMNS: &str = "
    e.id, e.status, e.mode, e.track, e.layout, e.ended_at, e.scheduled_at,
    e.name, e.description, e.writeup, e.allowed_vehicles, e.era_id,
    er.name as era_name
FROM events e
LEFT JOIN eras er ON e.era_id = er.id";

pub struct CreateMetronomeParams {
    pub track: String,
    pub layout: String,
    pub target_ms: i64,
    pub name: Option<String>,
    pub description: Option<String>,
    pub scheduled_at: Option<jiff::Timestamp>,
}

pub struct CreateShortcutParams {
    pub track: String,
    pub layout: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub scheduled_at: Option<jiff::Timestamp>,
}

pub struct CreateBombParams {
    pub track: String,
    pub layout: String,
    pub checkpoint_timeout_secs: i64,
    pub checkpoint_penalty_ms: i64,
    pub collision_max_penalty_ms: i64,
    pub name: Option<String>,
    pub description: Option<String>,
    pub scheduled_at: Option<jiff::Timestamp>,
}

pub struct UpdateEventParams<'a> {
    pub track: &'a str,
    pub layout: &'a str,
    pub name: Option<&'a str>,
    pub description: Option<&'a str>,
    pub scheduled_at: Option<jiff::Timestamp>,
    pub writeup: Option<&'a str>,
}

/// Returns filtered events with LIMIT/OFFSET pagination.
pub async fn filtered_events(
    pool: &Pool,
    status: Option<&str>,
    mode_filter: Option<&str>,
    era_id: Option<i64>,
    offset: i64,
    limit: i64,
) -> Result<Vec<Event>, sqlx::Error> {
    let mut qb = sqlx::QueryBuilder::<sqlx::Postgres>::new(format!("SELECT {EVENT_COLUMNS}"));
    append_event_filters(&mut qb, status, mode_filter, era_id);
    let _ = qb.push(" ORDER BY e.id DESC LIMIT ");
    let _ = qb.push_bind(limit);
    let _ = qb.push(" OFFSET ");
    let _ = qb.push_bind(offset);
    qb.build_query_as::<Event>().fetch_all(pool).await
}

/// Returns the total count matching the same filters as `filtered_events`.
pub async fn count_filtered_events(
    pool: &Pool,
    status: Option<&str>,
    mode_filter: Option<&str>,
    era_id: Option<i64>,
) -> Result<i64, sqlx::Error> {
    let mut qb = sqlx::QueryBuilder::<sqlx::Postgres>::new("SELECT COUNT(*) FROM events e");
    append_event_filters(&mut qb, status, mode_filter, era_id);
    let row: (i64,) = qb.build_query_as().fetch_one(pool).await?;
    Ok(row.0)
}

fn append_event_filters(
    qb: &mut sqlx::QueryBuilder<'_, sqlx::Postgres>,
    status: Option<&str>,
    mode_filter: Option<&str>,
    era_id: Option<i64>,
) {
    let has_status = matches!(status, Some("live") | Some("pending") | Some("completed"));
    let has_mode = mode_filter.is_some();
    let has_era = era_id.is_some();

    if has_status || has_mode || has_era {
        let _ = qb.push(" WHERE ");
        let mut first = true;
        if has_status {
            let _ = qb.push("e.status = ");
            let _ = qb.push_bind(status.unwrap().to_owned());
            first = false;
        }
        if has_mode {
            if !first {
                let _ = qb.push(" AND ");
            }
            let _ = qb.push("e.mode->>'type' = ");
            let _ = qb.push_bind(mode_filter.unwrap().to_owned());
            first = false;
        }
        if has_era {
            if !first {
                let _ = qb.push(" AND ");
            }
            let _ = qb.push("e.era_id = ");
            let _ = qb.push_bind(era_id.unwrap());
        }
    }
}

pub async fn get_event(pool: &Pool, id: i64) -> Result<Option<Event>, sqlx::Error> {
    sqlx::query_as(&format!("SELECT {EVENT_COLUMNS} WHERE e.id = $1"))
        .bind(id)
        .fetch_optional(pool)
        .await
}

/// Returns the current live event.
pub async fn ongoing_event(pool: &Pool) -> Result<Option<Event>, sqlx::Error> {
    sqlx::query_as(&format!(
        "SELECT {EVENT_COLUMNS}
         WHERE e.status = 'live'
         ORDER BY e.id DESC
         LIMIT 1"
    ))
    .fetch_optional(pool)
    .await
}

/// Returns the next pending events with a scheduled start in the future (max 4).
pub async fn upcoming_events(pool: &Pool) -> Result<Vec<Event>, sqlx::Error> {
    sqlx::query_as(&format!(
        "SELECT {EVENT_COLUMNS}
         WHERE e.status = 'pending' AND e.scheduled_at > NOW()
         ORDER BY e.scheduled_at ASC
         LIMIT 4"
    ))
    .fetch_all(pool)
    .await
}

pub async fn assign_event_era(
    pool: &Pool,
    event_id: i64,
    era_id: Option<i64>,
) -> Result<(), sqlx::Error> {
    let _ = sqlx::query("UPDATE events SET era_id = $1 WHERE id = $2")
        .bind(era_id)
        .bind(event_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn create_metronome_event(
    pool: &Pool,
    p: &CreateMetronomeParams,
) -> Result<i64, sqlx::Error> {
    let row = sqlx::query(
        "INSERT INTO events (mode, track, layout, created_at, name, description, scheduled_at)
         VALUES ($1, $2, $3, NOW(), $4, $5, $6)
         RETURNING id",
    )
    .bind(sqlx::types::Json(EventMode::Metronome {
        target_ms: p.target_ms,
    }))
    .bind(&p.track)
    .bind(&p.layout)
    .bind(p.name.as_deref())
    .bind(p.description.as_deref())
    .bind(p.scheduled_at.map(jiff_sqlx::Timestamp::from))
    .fetch_one(pool)
    .await?;
    Ok(row.get("id"))
}

pub async fn create_shortcut_event(
    pool: &Pool,
    p: &CreateShortcutParams,
) -> Result<i64, sqlx::Error> {
    let row = sqlx::query(
        "INSERT INTO events (mode, track, layout, created_at, name, description, scheduled_at)
         VALUES ($1, $2, $3, NOW(), $4, $5, $6)
         RETURNING id",
    )
    .bind(sqlx::types::Json(
        serde_json::to_value(EventMode::Shortcut).unwrap(),
    ))
    .bind(&p.track)
    .bind(&p.layout)
    .bind(p.name.as_deref())
    .bind(p.description.as_deref())
    .bind(p.scheduled_at.map(jiff_sqlx::Timestamp::from))
    .fetch_one(pool)
    .await?;
    Ok(row.get("id"))
}

pub async fn create_bomb_event(pool: &Pool, p: &CreateBombParams) -> Result<i64, sqlx::Error> {
    let mode = EventMode::Bomb {
        checkpoint_timeout_secs: p.checkpoint_timeout_secs,
        checkpoint_penalty_ms: p.checkpoint_penalty_ms,
        collision_max_penalty_ms: p.collision_max_penalty_ms,
    };
    let row = sqlx::query(
        "INSERT INTO events (mode, track, layout, created_at, name, description, scheduled_at)
         VALUES ($1, $2, $3, NOW(), $4, $5, $6)
         RETURNING id",
    )
    .bind(sqlx::types::Json(serde_json::to_value(mode).unwrap()))
    .bind(&p.track)
    .bind(&p.layout)
    .bind(p.name.as_deref())
    .bind(p.description.as_deref())
    .bind(p.scheduled_at.map(jiff_sqlx::Timestamp::from))
    .fetch_one(pool)
    .await?;
    Ok(row.get("id"))
}

pub async fn update_event_status(
    pool: &Pool,
    event_id: i64,
    status: EventStatus,
) -> Result<(), sqlx::Error> {
    let _ = sqlx::query("UPDATE events SET status = $1 WHERE id = $2")
        .bind(status)
        .bind(event_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn update_event(
    pool: &Pool,
    id: i64,
    p: &UpdateEventParams<'_>,
) -> Result<(), sqlx::Error> {
    let _ = sqlx::query(
        "UPDATE events SET track = $1, layout = $2, name = $3, description = $4,
                scheduled_at = $5, writeup = $6
         WHERE id = $7",
    )
    .bind(p.track)
    .bind(p.layout)
    .bind(p.name)
    .bind(p.description)
    .bind(p.scheduled_at.map(jiff_sqlx::Timestamp::from))
    .bind(p.writeup)
    .bind(id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn update_metronome_settings(
    pool: &Pool,
    event_id: i64,
    target_ms: i64,
) -> Result<(), sqlx::Error> {
    let _ = sqlx::query(
        "UPDATE events SET mode = jsonb_set(mode, '{target_ms}', $1::jsonb) WHERE id = $2",
    )
    .bind(serde_json::Value::Number(target_ms.into()))
    .bind(event_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn update_bomb_settings(
    pool: &Pool,
    event_id: i64,
    checkpoint_timeout_secs: i64,
) -> Result<(), sqlx::Error> {
    let _ = sqlx::query(
        "UPDATE events SET mode = jsonb_set(mode, '{checkpoint_timeout_secs}', $1::jsonb) WHERE id = $2",
    )
    .bind(serde_json::Value::Number(checkpoint_timeout_secs.into()))
    .bind(event_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn update_vehicle_restrictions(
    pool: &Pool,
    event_id: i64,
    vehicles: &[String],
) -> Result<(), sqlx::Error> {
    let _ = sqlx::query("UPDATE events SET allowed_vehicles = $1 WHERE id = $2")
        .bind(sqlx::types::Json(vehicles.to_vec()))
        .bind(event_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn all_eras(pool: &Pool) -> Result<Vec<Era>, sqlx::Error> {
    sqlx::query_as("SELECT id, name FROM eras ORDER BY id DESC")
        .fetch_all(pool)
        .await
}

pub async fn create_era(pool: &Pool, name: &str) -> Result<i64, sqlx::Error> {
    let row = sqlx::query("INSERT INTO eras (name) VALUES ($1) RETURNING id")
        .bind(name)
        .fetch_one(pool)
        .await?;
    Ok(row.get("id"))
}
