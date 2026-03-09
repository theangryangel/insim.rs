use askama::Template;
use axum::{
    extract::{Form, Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect},
};
use insim::core::track::Track;

use crate::db::{self, Session, SessionMode, SessionStatus};
use crate::web::state::{AppState, PageCtx};
use crate::web::filters;

// -- Shared types -------------------------------------------------------------

pub struct RoundResults {
    pub round: i64,
    pub results: Vec<db::MetronomeResult>,
}

pub fn group_metronome_rounds(results: Vec<db::MetronomeResult>) -> Vec<RoundResults> {
    let mut rounds: Vec<RoundResults> = Vec::new();
    for result in results {
        if let Some(last) = rounds.last_mut() {
            if last.round == result.round {
                last.results.push(result);
                continue;
            }
        }
        rounds.push(RoundResults { round: result.round, results: vec![result] });
    }
    rounds
}

// -- Template structs ---------------------------------------------------------

#[derive(Template)]
#[template(path = "sessions.html")]
pub struct SessionsTemplate {
    pub page: PageCtx,
    pub sessions: Vec<Session>,
}

#[derive(Template)]
#[template(path = "session_detail.html")]
pub struct SessionDetailTemplate {
    pub page: PageCtx,
    pub session: Session,
    pub metronome_standings: Vec<db::MetronomeStanding>,
    pub metronome_rounds: Vec<RoundResults>,
    pub round_results: Vec<(i64, Vec<db::MetronomeResult>)>,
    pub shortcut_best_times: Vec<db::ShortcutTime>,
    pub shortcut_all_times: Vec<db::ShortcutTime>,
    pub bomb_best_runs: Vec<db::BombRun>,
    pub bomb_all_runs: Vec<db::BombRun>,
}

#[derive(Template)]
#[template(path = "session_new.html")]
pub struct SessionNewTemplate {
    pub page: PageCtx,
    pub tracks: &'static [Track],
}

#[derive(Template)]
#[template(path = "session_edit.html")]
pub struct SessionEditTemplate {
    pub page: PageCtx,
    pub session: db::Session,
    pub tracks: &'static [Track],
}

#[derive(Template)]
#[template(path = "partials/session_actions.html")]
pub struct SessionActionsFragment {
    pub page: PageCtx,
    pub session: Session,
}

#[derive(Template)]
#[template(path = "partials/metronome_standings_content.html")]
pub struct MetronomeStandingsContent {
    pub metronome_standings: Vec<db::MetronomeStanding>,
}

#[derive(Template)]
#[template(path = "partials/metronome_round_content.html")]
pub struct MetronomeRoundContent {
    pub round_results: Vec<db::MetronomeResult>,
}

#[derive(Template)]
#[template(path = "partials/shortcut_standings.html")]
pub struct ShortcutStandingsFragment {
    pub session: Session,
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
    pub session: Session,
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
pub struct StartSessionForm {
    pub csrf_token: String,
}

fn default_rounds() -> i64 { 5 }
fn default_target() -> u64 { 20 }
fn default_max_scorers() -> i64 { 10 }
fn default_checkpoint_timeout() -> u64 { 30 }

#[derive(serde::Deserialize)]
pub struct NewSessionForm {
    pub csrf_token: String,
    pub mode: String,
    pub track: String,
    #[serde(default)]
    pub layout: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub scheduled_at: Option<String>,
    #[serde(default = "default_rounds")]
    pub rounds: i64,
    #[serde(default = "default_target")]
    pub target: u64,
    #[serde(default = "default_max_scorers")]
    pub max_scorers: i64,
    #[serde(default = "default_checkpoint_timeout")]
    pub checkpoint_timeout: u64,
}

#[derive(serde::Deserialize)]
pub struct EditSessionForm {
    pub csrf_token: String,
    pub track: String,
    #[serde(default)]
    pub layout: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub scheduled_at: Option<String>,
    pub writeup: Option<String>,
    pub rounds: Option<i64>,
    pub target: Option<u64>,
    pub max_scorers: Option<i64>,
    pub checkpoint_timeout: Option<u64>,
}

// -- Handlers -----------------------------------------------------------------

pub async fn sessions(
    page: PageCtx,
    State(state): State<AppState>,
) -> Result<Html<String>, StatusCode> {
    let sessions = db::all_sessions(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let tmpl = SessionsTemplate { page, sessions };
    Ok(Html(tmpl.render().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?))
}

pub async fn session_detail(
    page: PageCtx,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Html<String>, StatusCode> {
    let session = db::get_session(&state.pool, id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let mut metronome_standings = vec![];
    let mut metronome_rounds = vec![];
    let mut round_results: Vec<(i64, Vec<db::MetronomeResult>)> = vec![];
    let mut shortcut_best_times = vec![];
    let mut shortcut_all_times = vec![];
    let mut bomb_best_runs = vec![];
    let mut bomb_all_runs = vec![];

    match &*session.mode {
        SessionMode::Metronome { .. } => {
            metronome_standings = db::metronome_standings(&state.pool, session.id)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            let rounds = group_metronome_rounds(
                db::metronome_all_results(&state.pool, session.id)
                    .await
                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
            );
            round_results = rounds.iter().map(|r| (r.round, r.results.clone())).collect();
            metronome_rounds = rounds;
        }
        SessionMode::Shortcut => {
            shortcut_best_times = db::shortcut_best_times(&state.pool, session.id)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            shortcut_all_times = db::shortcut_all_times(&state.pool, session.id)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        }
        SessionMode::Bomb { .. } => {
            bomb_best_runs = db::bomb_best_runs(&state.pool, session.id)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            bomb_all_runs = db::bomb_all_runs(&state.pool, session.id)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        }
    };

    let tmpl = SessionDetailTemplate {
        page, session,
        metronome_standings, metronome_rounds, round_results,
        shortcut_best_times, shortcut_all_times,
        bomb_best_runs, bomb_all_runs,
    };
    Ok(Html(tmpl.render().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?))
}

pub async fn session_new_get(page: PageCtx) -> Result<Html<String>, StatusCode> {
    if !page.admin {
        return Err(StatusCode::NOT_FOUND);
    }
    let tmpl = SessionNewTemplate { page, tracks: Track::ALL };
    Ok(Html(tmpl.render().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?))
}

pub async fn session_new_post(
    page: PageCtx,
    State(state): State<AppState>,
    Form(form): Form<NewSessionForm>,
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

    let id = match form.mode.as_str() {
        "metronome" => {
            let target_ms = (form.target * 1000) as i64;
            db::create_metronome_session(
                &state.pool,
                &db::CreateMetronomeParams {
                    track,
                    layout: form.layout,
                    rounds: form.rounds,
                    target_ms,
                    max_scorers: form.max_scorers,
                    lobby_duration_secs: 300,
                    name,
                    description,
                    scheduled_at,
                },
            )
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        }
        "shortcut" => {
            db::create_shortcut_session(
                &state.pool,
                &db::CreateShortcutParams {
                    track,
                    layout: form.layout,
                    name,
                    description,
                    scheduled_at,
                },
            )
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        }
        "bomb" => {
            db::create_bomb_session(
                &state.pool,
                &db::CreateBombParams {
                    track,
                    layout: form.layout,
                    checkpoint_timeout_secs: form.checkpoint_timeout as i64,
                    name,
                    description,
                    scheduled_at,
                },
            )
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        }
        _ => return Err(StatusCode::BAD_REQUEST),
    };

    Ok(Redirect::to(&format!("/sessions/{id}")))
}

pub async fn session_edit_get(
    page: PageCtx,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Html<String>, StatusCode> {
    if !page.admin {
        return Err(StatusCode::NOT_FOUND);
    }
    let session = db::get_session(&state.pool, id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    let tmpl = SessionEditTemplate { page, session, tracks: Track::ALL };
    Ok(Html(tmpl.render().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?))
}

pub async fn session_edit_post(
    page: PageCtx,
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Form(form): Form<EditSessionForm>,
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
    let writeup = form.writeup.as_deref().filter(|s| !s.is_empty());

    db::update_session(
        &state.pool,
        id,
        &db::UpdateSessionParams { track, layout: &form.layout, name, description, scheduled_at, writeup },
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if let (Some(rounds), Some(target), Some(max_scorers)) =
        (form.rounds, form.target, form.max_scorers)
    {
        let target_ms = (target * 1000) as i64;
        db::update_metronome_settings(&state.pool, id, rounds, target_ms, max_scorers)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    if let Some(timeout) = form.checkpoint_timeout {
        db::update_bomb_settings(&state.pool, id, timeout as i64)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    Ok(Redirect::to(&format!("/sessions/{id}")))
}

pub async fn session_start(
    page: PageCtx,
    headers: axum::http::HeaderMap,
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Form(form): Form<StartSessionForm>,
) -> Result<axum::response::Response, StatusCode> {
    if !page.admin {
        return Err(StatusCode::NOT_FOUND);
    }
    if form.csrf_token != page.csrf_token {
        return Err(StatusCode::FORBIDDEN);
    }
    db::switch_session(&state.pool, id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if headers.contains_key("hx-request") {
        let session = db::get_session(&state.pool, id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .ok_or(StatusCode::NOT_FOUND)?;
        let html = SessionActionsFragment { page, session }
            .render()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok(Html(html).into_response())
    } else {
        Ok(Redirect::to("/").into_response())
    }
}

pub async fn session_cancel(
    page: PageCtx,
    headers: axum::http::HeaderMap,
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Form(form): Form<StartSessionForm>,
) -> Result<axum::response::Response, StatusCode> {
    if !page.admin {
        return Err(StatusCode::NOT_FOUND);
    }
    if form.csrf_token != page.csrf_token {
        return Err(StatusCode::FORBIDDEN);
    }
    db::cancel_session(&state.pool, id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if headers.contains_key("hx-request") {
        let session = db::get_session(&state.pool, id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .ok_or(StatusCode::NOT_FOUND)?;
        let html = SessionActionsFragment { page, session }
            .render()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok(Html(html).into_response())
    } else {
        Ok(Redirect::to(&format!("/sessions/{id}")).into_response())
    }
}

pub async fn session_standings(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Html<String>, StatusCode> {
    let session = db::get_session(&state.pool, id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let html = match &*session.mode {
        SessionMode::Metronome { .. } => {
            let metronome_standings = db::metronome_standings(&state.pool, session.id)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            MetronomeStandingsContent { metronome_standings }
                .render()
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        }
        SessionMode::Shortcut => {
            let shortcut_best_times = db::shortcut_best_times(&state.pool, session.id)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            let shortcut_all_times = db::shortcut_all_times(&state.pool, session.id)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            ShortcutStandingsFragment { session, shortcut_best_times, shortcut_all_times }
                .render()
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        }
        SessionMode::Bomb { .. } => {
            let bomb_best_runs = db::bomb_best_runs(&state.pool, session.id)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            let bomb_all_runs = db::bomb_all_runs(&state.pool, session.id)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            BombStandingsFragment { session, bomb_best_runs, bomb_all_runs }
                .render()
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        }
    };

    Ok(Html(html))
}

pub async fn session_round(
    State(state): State<AppState>,
    Path((id, round_number)): Path<(i64, i64)>,
) -> Result<Html<String>, StatusCode> {
    let metronome_rounds = group_metronome_rounds(
        db::metronome_all_results(&state.pool, id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
    );
    let round_results = metronome_rounds
        .iter()
        .find(|r| r.round == round_number)
        .map(|r| r.results.clone())
        .unwrap_or_default();
    Ok(Html(
        MetronomeRoundContent { round_results }
            .render()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
    ))
}

pub async fn session_standings_best(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Html<String>, StatusCode> {
    let shortcut_best_times = db::shortcut_best_times(&state.pool, id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Html(
        ShortcutBestTimesContent { shortcut_best_times }
            .render()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
    ))
}

pub async fn session_standings_all(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Html<String>, StatusCode> {
    let shortcut_all_times = db::shortcut_all_times(&state.pool, id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Html(
        ShortcutAllTimesContent { shortcut_all_times }
            .render()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
    ))
}

pub async fn session_bomb_best(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Html<String>, StatusCode> {
    let bomb_best_runs = db::bomb_best_runs(&state.pool, id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Html(
        BombBestRunsContent { bomb_best_runs }
            .render()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
    ))
}

pub async fn session_bomb_all(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Html<String>, StatusCode> {
    let bomb_all_runs = db::bomb_all_runs(&state.pool, id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Html(
        BombAllRunsContent { bomb_all_runs }
            .render()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
    ))
}
