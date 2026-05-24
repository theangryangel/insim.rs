use askama::Template;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect, Response},
};
use insim::core::track::Track;
use validator::Validate;

use super::{internal_error, parse_datetime_local};
use crate::{
    db::{self, EventMode, EventStatus},
    web::{
        AuthSession, Changeset, Csrf,
        state::{AppState, User},
    },
};

#[derive(Template)]
#[template(path = "event_edit.html")]
pub struct EventEditTemplate {
    pub current_user: User,
    pub csrf_token: String,
    pub event: db::Event,
    pub tracks: &'static [Track],
    pub eras: Vec<db::Era>,
    pub cs: Changeset<EditEventInput>,
}

#[derive(serde::Deserialize, Default, Validate)]
pub struct EditEventInput {
    pub track: Track,
    #[serde(default)]
    pub layout: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub scheduled_at: Option<String>,
    pub writeup: Option<String>,
    #[serde(default, deserialize_with = "crate::web::empty_string_as_none")]
    pub era_id: Option<i64>,
    pub era_name: Option<String>,
    #[serde(default, deserialize_with = "crate::web::empty_string_as_none")]
    pub target: Option<u64>,
    #[serde(default, deserialize_with = "crate::web::empty_string_as_none")]
    pub checkpoint_timeout: Option<u64>,
    #[serde(default)]
    pub allowed_vehicles: String,
    #[serde(default)]
    pub status: EventStatus,
}

pub async fn event_edit_get(
    auth: AuthSession,
    csrf: Csrf,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Html<String>, StatusCode> {
    let current_user = User::from(&auth);
    if !current_user.admin {
        return Err(StatusCode::NOT_FOUND);
    }
    let event = db::get_event(&state.pool, id)
        .await
        .map_err(internal_error)?
        .ok_or(StatusCode::NOT_FOUND)?;
    let eras = db::all_eras(&state.pool).await.map_err(internal_error)?;
    let allowed_vehicles = event
        .allowed_vehicles
        .0
        .iter()
        .cloned()
        .collect::<Vec<_>>()
        .join(", ");
    let (target, checkpoint_timeout) = match &*event.mode {
        EventMode::Metronome { target_ms } => (Some((target_ms / 1000) as u64), None),
        EventMode::Bomb {
            checkpoint_timeout_secs,
            ..
        } => (None, Some(*checkpoint_timeout_secs as u64)),
        EventMode::Shortcut => (None, None),
    };
    let cs = Changeset::new(EditEventInput {
        track: event.track,
        layout: event.layout.clone(),
        name: event.name.clone(),
        description: event.description.clone(),
        scheduled_at: event
            .scheduled_at
            .as_ref()
            .map(|ts| ts.to_jiff().strftime("%Y-%m-%dT%H:%M").to_string()),
        writeup: event.writeup.clone(),
        era_id: event.era_id,
        era_name: None,
        target,
        checkpoint_timeout,
        allowed_vehicles,
        status: event.status.clone(),
    });
    Ok(Html(
        EventEditTemplate {
            current_user,
            csrf_token: csrf.token,
            event,
            tracks: Track::ALL,
            eras,
            cs,
        }
        .render()
        .map_err(internal_error)?,
    ))
}

pub async fn event_edit_post(
    auth: AuthSession,
    csrf: Csrf,
    State(state): State<AppState>,
    Path(id): Path<i64>,
    cs: Changeset<EditEventInput>,
) -> Result<Response, StatusCode> {
    let current_user = User::from(&auth);
    if !current_user.admin {
        return Err(StatusCode::NOT_FOUND);
    }
    if !cs.is_valid() {
        let event = db::get_event(&state.pool, id)
            .await
            .map_err(internal_error)?
            .ok_or(StatusCode::NOT_FOUND)?;
        let eras = db::all_eras(&state.pool).await.map_err(internal_error)?;
        return Ok(EventEditTemplate {
            current_user,
            csrf_token: csrf.token,
            event,
            tracks: Track::ALL,
            eras,
            cs,
        }
        .render()
        .map_err(internal_error)
        .map(Html)?
        .into_response());
    }
    let name = cs.params.name.as_deref().filter(|s| !s.is_empty());
    let description = cs.params.description.as_deref().filter(|s| !s.is_empty());
    let scheduled_at = cs
        .params
        .scheduled_at
        .as_deref()
        .filter(|s| !s.is_empty())
        .map(parse_datetime_local)
        .transpose()?;
    let writeup = cs.params.writeup.as_deref().filter(|s| !s.is_empty());

    db::update_event(
        &state.pool,
        id,
        &db::UpdateEventParams {
            track: &cs.params.track.to_string(),
            layout: &cs.params.layout,
            name,
            description,
            scheduled_at,
            writeup,
        },
    )
    .await
    .map_err(internal_error)?;

    let era_id = if let Some(name) = cs.params.era_name.as_deref().filter(|s| !s.is_empty()) {
        Some(
            db::create_era(&state.pool, name)
                .await
                .map_err(internal_error)?,
        )
    } else {
        cs.params.era_id
    };
    db::assign_event_era(&state.pool, id, era_id)
        .await
        .map_err(internal_error)?;

    if let Some(target) = cs.params.target {
        db::update_metronome_settings(&state.pool, id, (target * 1000) as i64)
            .await
            .map_err(internal_error)?;
    }

    if let Some(timeout) = cs.params.checkpoint_timeout {
        db::update_bomb_settings(&state.pool, id, timeout as i64)
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
    db::update_vehicle_restrictions(&state.pool, id, &vehicles)
        .await
        .map_err(internal_error)?;

    db::update_event_status(&state.pool, id, cs.params.status.clone())
        .await
        .map_err(internal_error)?;

    Ok(Redirect::to(&format!("/events/{id}")).into_response())
}
