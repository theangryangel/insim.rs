use askama::Template;
use axum::{
    extract::{Form, Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect},
};
use insim::core::{track::Track, vehicle::Vehicle};

use crate::db::{self, Event, EventMode, EventStatus};
use crate::web::state::{AppState, PageCtx};
use crate::web::filters;

use super::internal_error;

// -- Template structs ---------------------------------------------------------

#[derive(Template)]
#[template(path = "events.html")]
pub struct EventsTemplate {
    pub page: PageCtx,
    pub events: Vec<Event>,
}

#[derive(Template)]
#[template(path = "event_detail.html")]
pub struct EventDetailTemplate {
    pub page: PageCtx,
    pub event: Event,
    pub metronome_standings: Vec<db::MetronomeStanding>,
    pub shortcut_best_times: Vec<db::ShortcutTime>,
    pub shortcut_all_times: Vec<db::ShortcutTime>,
    pub bomb_best_runs: Vec<db::BombRun>,
    pub bomb_all_runs: Vec<db::BombRun>,
}

#[derive(Template)]
#[template(path = "event_new.html")]
pub struct EventNewTemplate {
    pub page: PageCtx,
    pub tracks: &'static [Track],
}

#[derive(Template)]
#[template(path = "event_edit.html")]
pub struct EventEditTemplate {
    pub page: PageCtx,
    pub event: db::Event,
    pub tracks: &'static [Track],
    pub allowed_vehicles_str: String,
}

#[derive(Template)]
#[template(path = "partials/event_actions.html")]
pub struct EventActionsFragment {
    pub page: PageCtx,
    pub event: Event,
}

#[derive(Template)]
#[template(path = "partials/metronome_standings_content.html")]
pub struct MetronomeStandingsContent {
    pub metronome_standings: Vec<db::MetronomeStanding>,
}

#[derive(Template)]
#[template(path = "partials/shortcut_standings.html")]
pub struct ShortcutStandingsFragment {
    pub event: Event,
    pub shortcut_best_times: Vec<db::ShortcutTime>,
    pub shortcut_all_times: Vec<db::ShortcutTime>,
}

#[derive(Template)]
#[template(path = "partials/shortcut_best_times_content.html")]
pub struct ShortcutBestTimesContent {
    pub shortcut_best_times: Vec<db::ShortcutTime>,
}

#[derive(Template)]
#[template(path = "partials/shortcut_all_times_content.html")]
pub struct ShortcutAllTimesContent {
    pub shortcut_all_times: Vec<db::ShortcutTime>,
}

#[derive(Template)]
#[template(path = "partials/bomb_standings.html")]
pub struct BombStandingsFragment {
    pub event: Event,
    pub bomb_best_runs: Vec<db::BombRun>,
    pub bomb_all_runs: Vec<db::BombRun>,
}

#[derive(Template)]
#[template(path = "partials/bomb_best_runs_content.html")]
pub struct BombBestRunsContent {
    pub bomb_best_runs: Vec<db::BombRun>,
}

#[derive(Template)]
#[template(path = "partials/bomb_all_runs_content.html")]
pub struct BombAllRunsContent {
    pub bomb_all_runs: Vec<db::BombRun>,
}

// -- Form structs -------------------------------------------------------------

#[derive(serde::Deserialize)]
pub struct StartEventForm {
    pub csrf_token: String,
}

fn default_target() -> u64 { 20 }
fn default_checkpoint_timeout() -> u64 { 30 }

#[derive(serde::Deserialize)]
pub struct NewEventForm {
    pub csrf_token: String,
    pub mode: String,
    pub track: String,
    #[serde(default)]
    pub layout: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub scheduled_at: Option<String>,
    pub scheduled_end_at: Option<String>,
    #[serde(default = "default_target")]
    pub target: u64,
    #[serde(default = "default_checkpoint_timeout")]
    pub checkpoint_timeout: u64,
    #[serde(default)]
    pub allowed_vehicles: String,
}

#[derive(serde::Deserialize)]
pub struct EditEventForm {
    pub csrf_token: String,
    pub track: String,
    #[serde(default)]
    pub layout: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub scheduled_at: Option<String>,
    pub scheduled_end_at: Option<String>,
    pub writeup: Option<String>,
    pub target: Option<u64>,
    pub checkpoint_timeout: Option<u64>,
    #[serde(default)]
    pub allowed_vehicles: String,
}

// -- Helpers ------------------------------------------------------------------

fn parse_vehicles(vehicles_str: &str) -> Vec<Vehicle> {
    vehicles_str
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .filter_map(|s| s.parse::<Vehicle>().ok())
        .filter(|v| !matches!(v, Vehicle::Unknown))
        .collect()
}

fn event_vehicles_str(event: &db::Event) -> String {
    event.allowed_vehicles.0.iter()
        .map(|v| v.to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

// -- Handlers -----------------------------------------------------------------

pub async fn events(
    page: PageCtx,
    State(state): State<AppState>,
) -> Result<Html<String>, StatusCode> {
    let events = db::all_events(&state.pool)
        .await
        .map_err(internal_error)?;
    let tmpl = EventsTemplate { page, events };
    Ok(Html(tmpl.render().map_err(internal_error)?))
}

pub async fn event_detail(
    page: PageCtx,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Html<String>, StatusCode> {
    let event = db::get_event(&state.pool, id)
        .await
        .map_err(internal_error)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let mut metronome_standings = vec![];
    let mut shortcut_best_times = vec![];
    let mut shortcut_all_times = vec![];
    let mut bomb_best_runs = vec![];
    let mut bomb_all_runs = vec![];

    match &*event.mode {
        EventMode::Metronome { .. } => {
            metronome_standings = db::metronome_standings(&state.pool, event.id)
                .await
                .map_err(internal_error)?;
        }
        EventMode::Shortcut => {
            shortcut_best_times = db::shortcut_best_times(&state.pool, event.id)
                .await
                .map_err(internal_error)?;
            shortcut_all_times = db::shortcut_all_times(&state.pool, event.id)
                .await
                .map_err(internal_error)?;
        }
        EventMode::Bomb { .. } => {
            bomb_best_runs = db::bomb_best_runs(&state.pool, event.id)
                .await
                .map_err(internal_error)?;
            bomb_all_runs = db::bomb_all_runs(&state.pool, event.id)
                .await
                .map_err(internal_error)?;
        }
    };

    let tmpl = EventDetailTemplate {
        page, event,
        metronome_standings,
        shortcut_best_times, shortcut_all_times,
        bomb_best_runs, bomb_all_runs,
    };
    Ok(Html(tmpl.render().map_err(internal_error)?))
}

pub async fn event_new_get(page: PageCtx) -> Result<Html<String>, StatusCode> {
    if !page.admin {
        return Err(StatusCode::NOT_FOUND);
    }
    let tmpl = EventNewTemplate { page, tracks: Track::ALL };
    Ok(Html(tmpl.render().map_err(internal_error)?))
}

pub async fn event_new_post(
    page: PageCtx,
    State(state): State<AppState>,
    Form(form): Form<NewEventForm>,
) -> Result<Redirect, StatusCode> {
    if !page.admin {
        return Err(StatusCode::NOT_FOUND);
    }
    if form.csrf_token != page.csrf_token {
        return Err(StatusCode::FORBIDDEN);
    }
    let track = form.track.parse().map_err(|_| StatusCode::BAD_REQUEST)?;
    let name = form.name.filter(|s| !s.is_empty());
    let description = form.description.filter(|s| !s.is_empty());
    let scheduled_at = form.scheduled_at.filter(|s| !s.is_empty());
    let scheduled_end_at = form.scheduled_end_at.filter(|s| !s.is_empty());

    if let (Some(start), Some(end)) = (scheduled_at.as_deref(), scheduled_end_at.as_deref()) {
        let overlap = db::has_scheduling_overlap(&state.pool, start, end, None)
            .await
            .map_err(internal_error)?;
        if overlap {
            return Err(StatusCode::CONFLICT);
        }
    }

    let id = match form.mode.as_str() {
        "metronome" => {
            let target_ms = (form.target * 1000) as i64;
            db::create_metronome_event(
                &state.pool,
                &db::CreateMetronomeParams {
                    track,
                    layout: form.layout,
                    target_ms,
                    name,
                    description,
                    scheduled_at,
                    scheduled_end_at,
                },
            )
            .await
            .map_err(internal_error)?
        }
        "shortcut" => {
            db::create_shortcut_event(
                &state.pool,
                &db::CreateShortcutParams {
                    track,
                    layout: form.layout,
                    name,
                    description,
                    scheduled_at,
                    scheduled_end_at,
                },
            )
            .await
            .map_err(internal_error)?
        }
        "bomb" => {
            db::create_bomb_event(
                &state.pool,
                &db::CreateBombParams {
                    track,
                    layout: form.layout,
                    checkpoint_timeout_secs: form.checkpoint_timeout as i64,
                    name,
                    description,
                    scheduled_at,
                    scheduled_end_at,
                },
            )
            .await
            .map_err(internal_error)?
        }
        _ => return Err(StatusCode::BAD_REQUEST),
    };

    let vehicles = parse_vehicles(&form.allowed_vehicles);
    if !vehicles.is_empty() {
        db::update_vehicle_restrictions(&state.pool, id, &vehicles)
            .await
            .map_err(internal_error)?;
    }

    Ok(Redirect::to(&format!("/events/{id}")))
}

pub async fn event_edit_get(
    page: PageCtx,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Html<String>, StatusCode> {
    if !page.admin {
        return Err(StatusCode::NOT_FOUND);
    }
    let event = db::get_event(&state.pool, id)
        .await
        .map_err(internal_error)?
        .ok_or(StatusCode::NOT_FOUND)?;
    let allowed_vehicles_str = event_vehicles_str(&event);
    let tmpl = EventEditTemplate { page, event, tracks: Track::ALL, allowed_vehicles_str };
    Ok(Html(tmpl.render().map_err(internal_error)?))
}

pub async fn event_edit_post(
    page: PageCtx,
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Form(form): Form<EditEventForm>,
) -> Result<Redirect, StatusCode> {
    if !page.admin {
        return Err(StatusCode::NOT_FOUND);
    }
    if form.csrf_token != page.csrf_token {
        return Err(StatusCode::FORBIDDEN);
    }
    let track = form.track.parse().map_err(|_| StatusCode::BAD_REQUEST)?;
    let name = form.name.as_deref().filter(|s| !s.is_empty());
    let description = form.description.as_deref().filter(|s| !s.is_empty());
    let scheduled_at = form.scheduled_at.as_deref().filter(|s| !s.is_empty());
    let scheduled_end_at = form.scheduled_end_at.as_deref().filter(|s| !s.is_empty());
    let writeup = form.writeup.as_deref().filter(|s| !s.is_empty());

    if let (Some(start), Some(end)) = (scheduled_at, scheduled_end_at) {
        let overlap = db::has_scheduling_overlap(&state.pool, start, end, Some(id))
            .await
            .map_err(internal_error)?;
        if overlap {
            return Err(StatusCode::CONFLICT);
        }
    }

    db::update_event(
        &state.pool,
        id,
        &db::UpdateEventParams { track, layout: &form.layout, name, description, scheduled_at, scheduled_end_at, writeup },
    )
    .await
    .map_err(internal_error)?;

    if let Some(target) = form.target {
        let target_ms = (target * 1000) as i64;
        db::update_metronome_settings(&state.pool, id, target_ms)
            .await
            .map_err(internal_error)?;
    }

    if let Some(timeout) = form.checkpoint_timeout {
        db::update_bomb_settings(&state.pool, id, timeout as i64)
            .await
            .map_err(internal_error)?;
    }

    let vehicles = parse_vehicles(&form.allowed_vehicles);
    db::update_vehicle_restrictions(&state.pool, id, &vehicles)
        .await
        .map_err(internal_error)?;

    Ok(Redirect::to(&format!("/events/{id}")))
}

pub async fn event_start(
    page: PageCtx,
    headers: axum::http::HeaderMap,
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Form(form): Form<StartEventForm>,
) -> Result<axum::response::Response, StatusCode> {
    if !page.admin {
        return Err(StatusCode::NOT_FOUND);
    }
    if form.csrf_token != page.csrf_token {
        return Err(StatusCode::FORBIDDEN);
    }
    db::switch_event(&state.pool, id)
        .await
        .map_err(internal_error)?;
    if headers.contains_key("hx-request") {
        let event = db::get_event(&state.pool, id)
            .await
            .map_err(internal_error)?
            .ok_or(StatusCode::NOT_FOUND)?;
        let html = EventActionsFragment { page, event }
            .render()
            .map_err(internal_error)?;
        Ok(Html(html).into_response())
    } else {
        Ok(Redirect::to("/").into_response())
    }
}

pub async fn event_cancel(
    page: PageCtx,
    headers: axum::http::HeaderMap,
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Form(form): Form<StartEventForm>,
) -> Result<axum::response::Response, StatusCode> {
    if !page.admin {
        return Err(StatusCode::NOT_FOUND);
    }
    if form.csrf_token != page.csrf_token {
        return Err(StatusCode::FORBIDDEN);
    }
    db::cancel_event(&state.pool, id)
        .await
        .map_err(internal_error)?;
    if headers.contains_key("hx-request") {
        let event = db::get_event(&state.pool, id)
            .await
            .map_err(internal_error)?
            .ok_or(StatusCode::NOT_FOUND)?;
        let html = EventActionsFragment { page, event }
            .render()
            .map_err(internal_error)?;
        Ok(Html(html).into_response())
    } else {
        Ok(Redirect::to(&format!("/events/{id}")).into_response())
    }
}

pub async fn event_complete(
    page: PageCtx,
    headers: axum::http::HeaderMap,
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Form(form): Form<StartEventForm>,
) -> Result<axum::response::Response, StatusCode> {
    if !page.admin {
        return Err(StatusCode::NOT_FOUND);
    }
    if form.csrf_token != page.csrf_token {
        return Err(StatusCode::FORBIDDEN);
    }
    db::complete_event(&state.pool, id)
        .await
        .map_err(internal_error)?;
    if headers.contains_key("hx-request") {
        let event = db::get_event(&state.pool, id)
            .await
            .map_err(internal_error)?
            .ok_or(StatusCode::NOT_FOUND)?;
        let html = EventActionsFragment { page, event }
            .render()
            .map_err(internal_error)?;
        Ok(Html(html).into_response())
    } else {
        Ok(Redirect::to(&format!("/events/{id}")).into_response())
    }
}

pub async fn event_standings(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Html<String>, StatusCode> {
    let event = db::get_event(&state.pool, id)
        .await
        .map_err(internal_error)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let html = match &*event.mode {
        EventMode::Metronome { .. } => {
            let metronome_standings = db::metronome_standings(&state.pool, event.id)
                .await
                .map_err(internal_error)?;
            MetronomeStandingsContent { metronome_standings }
                .render()
                .map_err(internal_error)?
        }
        EventMode::Shortcut => {
            let shortcut_best_times = db::shortcut_best_times(&state.pool, event.id)
                .await
                .map_err(internal_error)?;
            let shortcut_all_times = db::shortcut_all_times(&state.pool, event.id)
                .await
                .map_err(internal_error)?;
            ShortcutStandingsFragment { event, shortcut_best_times, shortcut_all_times }
                .render()
                .map_err(internal_error)?
        }
        EventMode::Bomb { .. } => {
            let bomb_best_runs = db::bomb_best_runs(&state.pool, event.id)
                .await
                .map_err(internal_error)?;
            let bomb_all_runs = db::bomb_all_runs(&state.pool, event.id)
                .await
                .map_err(internal_error)?;
            BombStandingsFragment { event, bomb_best_runs, bomb_all_runs }
                .render()
                .map_err(internal_error)?
        }
    };

    Ok(Html(html))
}

pub async fn event_standings_best(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Html<String>, StatusCode> {
    let shortcut_best_times = db::shortcut_best_times(&state.pool, id)
        .await
        .map_err(internal_error)?;
    Ok(Html(
        ShortcutBestTimesContent { shortcut_best_times }
            .render()
            .map_err(internal_error)?,
    ))
}

pub async fn event_standings_all(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Html<String>, StatusCode> {
    let shortcut_all_times = db::shortcut_all_times(&state.pool, id)
        .await
        .map_err(internal_error)?;
    Ok(Html(
        ShortcutAllTimesContent { shortcut_all_times }
            .render()
            .map_err(internal_error)?,
    ))
}

pub async fn event_bomb_best(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Html<String>, StatusCode> {
    let bomb_best_runs = db::bomb_best_runs(&state.pool, id)
        .await
        .map_err(internal_error)?;
    Ok(Html(
        BombBestRunsContent { bomb_best_runs }
            .render()
            .map_err(internal_error)?,
    ))
}

pub async fn event_bomb_all(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Html<String>, StatusCode> {
    let bomb_all_runs = db::bomb_all_runs(&state.pool, id)
        .await
        .map_err(internal_error)?;
    Ok(Html(
        BombAllRunsContent { bomb_all_runs }
            .render()
            .map_err(internal_error)?,
    ))
}
