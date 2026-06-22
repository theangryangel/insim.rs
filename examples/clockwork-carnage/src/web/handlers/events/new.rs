use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Redirect, Response},
};
use insim::core::track::Track;
use validator::Validate;

use super::{internal_error, parse_datetime_local};
use crate::{
    db::{self, EventStatus},
    web::{
        AuthSession, Changeset, Csrf,
        state::{AppState, User},
        views,
    },
};

#[derive(serde::Deserialize, Validate)]
pub struct NewEventInput {
    #[validate(length(min = 1, message = "Select a mode"))]
    pub mode: String,
    #[serde(default, deserialize_with = "crate::web::empty_string_as_none")]
    #[validate(required(message = "Required"))]
    pub track: Option<Track>,
    #[serde(default)]
    pub layout: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub scheduled_at: Option<String>,
    #[serde(default, deserialize_with = "crate::web::empty_string_as_none")]
    pub era_id: Option<i64>,
    pub era_name: Option<String>,
    #[serde(default = "default_target")]
    pub target: u64,
    #[serde(default = "default_checkpoint_timeout")]
    pub checkpoint_timeout: u64,
    #[serde(default = "default_checkpoint_penalty_ms")]
    pub checkpoint_penalty_ms: u64,
    #[serde(default = "default_collision_max_penalty_ms")]
    pub collision_max_penalty_ms: u64,
    #[serde(default)]
    pub allowed_vehicles: String,
    #[serde(default)]
    pub status: EventStatus,
    /// phx-change marker: `"change"` = live re-render (no commit).
    #[serde(default, rename = "_event")]
    pub event: Option<String>,
}

fn default_target() -> u64 {
    20
}
fn default_checkpoint_timeout() -> u64 {
    30
}
fn default_checkpoint_penalty_ms() -> u64 {
    250
}
fn default_collision_max_penalty_ms() -> u64 {
    500
}

impl Default for NewEventInput {
    fn default() -> Self {
        Self {
            mode: "metronome".to_string(),
            track: None,
            layout: String::new(),
            name: None,
            description: None,
            scheduled_at: None,
            era_id: None,
            era_name: None,
            target: default_target(),
            checkpoint_timeout: default_checkpoint_timeout(),
            checkpoint_penalty_ms: default_checkpoint_penalty_ms(),
            collision_max_penalty_ms: default_collision_max_penalty_ms(),
            allowed_vehicles: String::new(),
            status: EventStatus::Pending,
            event: None,
        }
    }
}

pub async fn event_new_get(
    auth: AuthSession,
    csrf: Csrf,
    State(state): State<AppState>,
) -> Result<Html<String>, StatusCode> {
    let current_user = User::from(&auth);
    if !current_user.admin {
        return Err(StatusCode::NOT_FOUND);
    }
    let eras = db::all_eras(&state.pool).await.map_err(internal_error)?;
    let cs = Changeset::empty();
    Ok(Html(
        views::event_new(&current_user, &csrf.token, Track::ALL, &eras, &cs).into_string(),
    ))
}

pub async fn event_new_post(
    auth: AuthSession,
    csrf: Csrf,
    State(state): State<AppState>,
    cs: Changeset<NewEventInput>,
) -> Result<Response, StatusCode> {
    let current_user = User::from(&auth);
    if !current_user.admin {
        return Err(StatusCode::NOT_FOUND);
    }

    // phx-change: re-render the form from the changeset; the client morphs it
    // back in via Alpine. No commit, no side effects.
    if cs.params.event.as_deref() == Some("change") {
        let eras = db::all_eras(&state.pool).await.map_err(internal_error)?;
        return Ok(
            Html(views::event_new_form(&csrf.token, Track::ALL, &eras, &cs).into_string())
                .into_response(),
        );
    }

    if !cs.is_valid() {
        let eras = db::all_eras(&state.pool).await.map_err(internal_error)?;
        return Ok(Html(
            views::event_new(&current_user, &csrf.token, Track::ALL, &eras, &cs).into_string(),
        )
        .into_response());
    }
    let name = cs
        .params
        .name
        .as_deref()
        .filter(|s| !s.is_empty())
        .map(str::to_string);
    let description = cs
        .params
        .description
        .as_deref()
        .filter(|s| !s.is_empty())
        .map(str::to_string);
    let scheduled_at = cs
        .params
        .scheduled_at
        .as_deref()
        .filter(|s| !s.is_empty())
        .map(parse_datetime_local)
        .transpose()?;

    let id = match cs.params.mode.as_str() {
        "metronome" => db::create_metronome_event(
            &state.pool,
            &db::CreateMetronomeParams {
                track: cs.params.track.expect("validated").to_string(),
                layout: cs.params.layout.clone(),
                target_ms: (cs.params.target * 1000) as i64,
                name,
                description,
                scheduled_at,
            },
        )
        .await
        .map_err(internal_error)?,
        "shortcut" => db::create_shortcut_event(
            &state.pool,
            &db::CreateShortcutParams {
                track: cs.params.track.expect("validated").to_string(),
                layout: cs.params.layout.clone(),
                name,
                description,
                scheduled_at,
            },
        )
        .await
        .map_err(internal_error)?,
        "bomb" => db::create_bomb_event(
            &state.pool,
            &db::CreateBombParams {
                track: cs.params.track.expect("validated").to_string(),
                layout: cs.params.layout.clone(),
                checkpoint_timeout_secs: cs.params.checkpoint_timeout as i64,
                checkpoint_penalty_ms: cs.params.checkpoint_penalty_ms as i64,
                collision_max_penalty_ms: cs.params.collision_max_penalty_ms as i64,
                name,
                description,
                scheduled_at,
            },
        )
        .await
        .map_err(internal_error)?,
        _ => return Err(StatusCode::BAD_REQUEST),
    };

    let era_id = if let Some(name) = cs.params.era_name.as_deref().filter(|s| !s.is_empty()) {
        Some(
            db::create_era(&state.pool, name)
                .await
                .map_err(internal_error)?,
        )
    } else {
        cs.params.era_id
    };
    if era_id.is_some() {
        db::assign_event_era(&state.pool, id, era_id)
            .await
            .map_err(internal_error)?;
    }

    let vehicles: Vec<String> = cs
        .params
        .allowed_vehicles
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    if !vehicles.is_empty() {
        db::update_vehicle_restrictions(&state.pool, id, &vehicles)
            .await
            .map_err(internal_error)?;
    }

    db::update_event_status(&state.pool, id, cs.params.status.clone())
        .await
        .map_err(internal_error)?;

    Ok(Redirect::to(&format!("/events/{id}")).into_response())
}
