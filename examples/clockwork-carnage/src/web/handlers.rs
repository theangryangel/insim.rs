use axum::{
    extract::{Form, Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect},
};
use askama::Template;
use insim::core::track::Track;
use oauth2::{CsrfToken, Scope};
use tower_sessions::Session as TowerSession;

use crate::db::{self, SessionMode};
use crate::web::{AuthSession, OAuthCredentials};
use crate::web::state::{AppState, PageCtx};
use crate::web::templates::{
    BombAllRunsFragment, BombBestRunsFragment, BombStandingsFragment,
    IndexTemplate, MetronomeRoundTab, MetronomeStandingsTab,
    SessionActionsFragment, SessionDetailTemplate, SessionEditTemplate,
    SessionNewTemplate, SessionsTemplate, ShortcutAllTimesFragment,
    ShortcutBestTimesFragment, ShortcutStandingsFragment,
    group_metronome_rounds,
};

// -- Static assets ------------------------------------------------------------

pub async fn logo() -> (StatusCode, [(&'static str, &'static str); 1], &'static [u8]) {
    (
        StatusCode::OK,
        [("content-type", "image/svg+xml")],
        include_bytes!("../../logo.svg"),
    )
}

// -- Page handlers ------------------------------------------------------------

pub async fn index(
    page: PageCtx,
    State(state): State<AppState>,
) -> Result<Html<String>, StatusCode> {
    let active = db::active_session(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let upcoming = db::upcoming_sessions(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let tmpl = IndexTemplate { page, active, upcoming };
    Ok(Html(tmpl.render().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?))
}

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

    let (metronome_standings, metronome_rounds, shortcut_best_times, bomb_best_runs) = match &*session.mode {
        SessionMode::Metronome { .. } => {
            let standings = db::metronome_standings(&state.pool, session.id)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            let rounds = group_metronome_rounds(
                db::metronome_all_results(&state.pool, session.id)
                    .await
                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
            );
            (standings, rounds, vec![], vec![])
        }
        SessionMode::Shortcut => {
            let t = db::shortcut_best_times(&state.pool, session.id)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            (vec![], vec![], t, vec![])
        }
        SessionMode::Bomb { .. } => {
            let b = db::bomb_best_runs(&state.pool, session.id)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            (vec![], vec![], vec![], b)
        }
    };

    let tmpl = SessionDetailTemplate { page, session, metronome_standings, metronome_rounds, shortcut_best_times, bomb_best_runs };
    Ok(Html(tmpl.render().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?))
}

#[derive(serde::Deserialize)]
pub struct StartSessionForm {
    pub csrf_token: String,
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

    Ok(Redirect::to(&format!("/sessions/{id}")))
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
            let metronome_rounds = group_metronome_rounds(
                db::metronome_all_results(&state.pool, session.id)
                    .await
                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
            );
            MetronomeStandingsTab { session, metronome_standings, metronome_rounds }
                .render()
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        }
        SessionMode::Shortcut => {
            let shortcut_best_times = db::shortcut_best_times(&state.pool, session.id)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            ShortcutStandingsFragment { session, shortcut_best_times }
                .render()
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        }
        SessionMode::Bomb { .. } => {
            let bomb_best_runs = db::bomb_best_runs(&state.pool, session.id)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            BombStandingsFragment { session, bomb_best_runs }
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
    let session = db::get_session(&state.pool, id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
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
        MetronomeRoundTab { session, round_number, round_results, metronome_rounds }
            .render()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
    ))
}

pub async fn session_standings_best(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Html<String>, StatusCode> {
    let session = db::get_session(&state.pool, id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    let shortcut_best_times = db::shortcut_best_times(&state.pool, id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Html(
        ShortcutBestTimesFragment { session, shortcut_best_times }
            .render()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
    ))
}

pub async fn session_standings_all(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Html<String>, StatusCode> {
    let session = db::get_session(&state.pool, id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    let shortcut_all_times = db::shortcut_all_times(&state.pool, id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Html(
        ShortcutAllTimesFragment { session, shortcut_all_times }
            .render()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
    ))
}

pub async fn session_bomb_best(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Html<String>, StatusCode> {
    let session = db::get_session(&state.pool, id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    let bomb_best_runs = db::bomb_best_runs(&state.pool, id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Html(
        BombBestRunsFragment { session, bomb_best_runs }
            .render()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
    ))
}

pub async fn session_bomb_all(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Html<String>, StatusCode> {
    let session = db::get_session(&state.pool, id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    let bomb_all_runs = db::bomb_all_runs(&state.pool, id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Html(
        BombAllRunsFragment { session, bomb_all_runs }
            .render()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
    ))
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

// -- Auth handlers ------------------------------------------------------------

pub async fn login(
    auth_session: AuthSession,
    session: TowerSession,
    State(state): State<AppState>,
) -> impl IntoResponse {
    if auth_session.user.is_some() {
        return Redirect::to("/").into_response();
    }
    let (auth_url, csrf_token) = state
        .oauth_client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("openid".to_string()))
        .add_scope(Scope::new("profile".to_string()))
        .url();
    session
        .insert("oauth_csrf_state", csrf_token.secret().clone())
        .await
        .unwrap();
    Redirect::to(auth_url.as_str()).into_response()
}

#[derive(serde::Deserialize)]
pub struct AuthzResp {
    pub code: String,
    pub state: String,
}

pub async fn callback(
    mut auth_session: AuthSession,
    session: TowerSession,
    Query(params): Query<AuthzResp>,
) -> impl IntoResponse {
    let expected: Option<String> = session.remove("oauth_csrf_state").await.unwrap();
    if expected.as_deref() != Some(&params.state) {
        return StatusCode::BAD_REQUEST.into_response();
    }
    let creds = OAuthCredentials { code: params.code, state: params.state };
    match auth_session.authenticate(creds).await {
        Ok(Some(user)) => {
            auth_session.login(&user).await.unwrap();
            Redirect::to("/").into_response()
        }
        Ok(None) => StatusCode::UNAUTHORIZED.into_response(),
        Err(e) => {
            tracing::error!("Auth error: {e}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub async fn logout(mut auth_session: AuthSession) -> impl IntoResponse {
    let _ = auth_session.logout().await.unwrap();
    Redirect::to("/")
}
