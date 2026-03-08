//! Clockwork Carnage — unified binary (InSim runner + web dashboard).

#![allow(missing_docs, missing_debug_implementations)]

mod db;
mod games;
mod hud;
mod web;

type ChatError = kitcar::chat::RuntimeError;
const MIN_PLAYERS: usize = 2;

use std::net::SocketAddr;
use std::sync::Arc;

use askama::Template;
use axum::{
    Router,
    extract::{Form, FromRequestParts, Path, Query, State},
    http::{StatusCode, request::Parts},
    response::{Html, IntoResponse, Redirect},
    routing::{get, post},
};
use axum_login::AuthManagerLayerBuilder;
use clap::{Parser, Subcommand};
use db::{Session, SessionMode, SessionStatus};
use games::{GameCtx, execute};
use web::{AuthSession, Backend, OAuthCredentials};
use insim::{WithRequestId, core::track::Track, insim::TinyType};
use kitcar::{game, presence};
use oauth2::{AuthUrl, ClientId, CsrfToken, RedirectUrl, Scope, basic::BasicClient};
use sqlx::types::Json;
use tower_sessions::{SessionManagerLayer, Session as TowerSession, cookie::Key, cookie::SameSite};
use tower_sessions_sqlx_store::SqliteStore;

// -- CLI ----------------------------------------------------------------------

#[derive(Debug, Parser)]
struct Args {
    #[arg(long, default_value = "clockwork-carnage.db")]
    db: String,

    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Start the runner (connects to InSim, polls for active sessions, serves web UI)
    Run {
        #[arg(short, long)]
        addr: SocketAddr,

        #[arg(short, long)]
        password: Option<String>,

        #[arg(long, default_value = "127.0.0.1:3000")]
        listen: SocketAddr,
    },

    /// Queue a new session
    Add {
        #[command(subcommand)]
        mode: AddMode,
    },

    /// List all sessions
    List,

    /// Activate a pending session (sets status to ACTIVE)
    Activate {
        /// Session ID to activate
        id: i64,
    },

    /// Set the post-event write-up for a session
    Writeup {
        /// Session ID
        id: i64,

        /// Write-up text
        text: String,
    },
}

#[derive(Debug, Subcommand)]
enum AddMode {
    /// Create a metronome (event) session
    Metronome {
        #[arg(short, long)]
        track: Track,

        #[arg(short, long, default_value = "")]
        layout: String,

        #[arg(short, long, default_value_t = 5)]
        rounds: i64,

        #[arg(long, default_value_t = 20)]
        target: u64,

        #[arg(short, long, default_value_t = 10)]
        max_scorers: i64,

        #[arg(long, default_value_t = 300)]
        lobby_duration_secs: u64,

        #[arg(long)]
        name: Option<String>,

        #[arg(long)]
        description: Option<String>,

        #[arg(long)]
        scheduled_at: Option<String>,
    },

    /// Create a shortcut (challenge) session
    Shortcut {
        #[arg(short, long)]
        track: Track,

        #[arg(short, long, default_value = "")]
        layout: String,

        #[arg(long)]
        name: Option<String>,

        #[arg(long)]
        description: Option<String>,

        #[arg(long)]
        scheduled_at: Option<String>,
    },
}

// -- Web: filters -------------------------------------------------------------

mod filters {
    use insim::core::string::{colours::Colour, escaping::Escape};

    #[askama::filter_fn]
    pub fn format_time_ms(ms: &i64, _env: &dyn askama::Values) -> askama::Result<String> {
        let ms = *ms;
        let minutes = ms / 60_000;
        let seconds = (ms % 60_000) / 1000;
        let millis = ms % 1000;
        Ok(format!("{minutes}:{seconds:02}.{millis:03}"))
    }

    #[askama::filter_fn]
    pub fn format_delta_ms(ms: &i64, _env: &dyn askama::Values) -> askama::Result<String> {
        let ms = *ms;
        let total = ms.abs();
        let sign = if ms >= 0 { "+" } else { "-" };
        let seconds = total / 1000;
        let millis = total % 1000;
        Ok(format!("{sign}{seconds}.{millis:03}s"))
    }

    /// Convert an LFS player name with colour markers into HTML `<span>` elements.
    /// Must be used with `|safe` in templates.
    #[askama::filter_fn]
    pub fn colour_html(s: &str, _env: &dyn askama::Values) -> askama::Result<String> {
        fn lfs_colour_class(c: u8) -> &'static str {
            match c {
                0 => "text-gray-900",
                1 => "text-red-500",
                2 => "text-green-500",
                3 => "text-amber-500",
                4 => "text-blue-500",
                5 => "text-purple-500",
                6 => "text-cyan-500",
                _ => "", // 7 (white) and 8 (default): inherit parent colour
            }
        }

        fn html_escape(s: &str) -> String {
            s.chars().fold(String::with_capacity(s.len()), |mut out, c| {
                match c {
                    '&'  => out.push_str("&amp;"),
                    '<'  => out.push_str("&lt;"),
                    '>'  => out.push_str("&gt;"),
                    '"'  => out.push_str("&quot;"),
                    '\'' => out.push_str("&#39;"),
                    c    => out.push(c),
                }
                out
            })
        }

        let mut out = String::new();
        for (colour, chunk) in s.colour_spans() {
            let text = html_escape(&chunk.unescape());
            let class = lfs_colour_class(colour);
            if class.is_empty() {
                out.push_str(&text);
            } else {
                out.push_str(&format!("<span class=\"{class}\">{text}</span>"));
            }
        }
        Ok(out)
    }
}

// -- Web: templates -----------------------------------------------------------

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    page: PageCtx,
    active: Option<Session>,
    upcoming: Vec<Session>,
}

#[derive(Template)]
#[template(path = "sessions.html")]
struct SessionsTemplate {
    page: PageCtx,
    sessions: Vec<Session>,
}

struct RoundResults {
    round: i64,
    results: Vec<db::MetronomeResult>,
}

#[derive(Template)]
#[template(path = "session_detail.html")]
struct SessionDetailTemplate {
    page: PageCtx,
    session: Session,
    metronome_standings: Vec<db::MetronomeStanding>,
    metronome_rounds: Vec<RoundResults>,
    shortcut_best_times: Vec<db::ShortcutTime>,
}

#[derive(Template)]
#[template(path = "session_new.html")]
struct SessionNewTemplate {
    page: PageCtx,
    tracks: &'static [Track],
}

#[derive(Template)]
#[template(path = "session_edit.html")]
struct SessionEditTemplate {
    page: PageCtx,
    session: db::Session,
    tracks: &'static [Track],
}

#[derive(Template)]
#[template(path = "partials/session_actions.html")]
struct SessionActionsFragment {
    page: PageCtx,
    session: Session,
}

#[derive(Template)]
#[template(path = "partials/metronome_standings_response.html")]
struct MetronomeStandingsTab {
    session: Session,
    metronome_standings: Vec<db::MetronomeStanding>,
    metronome_rounds: Vec<RoundResults>,
}

#[derive(Template)]
#[template(path = "partials/metronome_round_response.html")]
struct MetronomeRoundTab {
    session: Session,
    round_number: i64,
    round_results: Vec<db::MetronomeResult>,
    metronome_rounds: Vec<RoundResults>,
}

#[derive(Template)]
#[template(path = "partials/shortcut_standings.html")]
struct ShortcutStandingsFragment {
    session: Session,
    shortcut_best_times: Vec<db::ShortcutTime>,
}

#[derive(Template)]
#[template(path = "partials/shortcut_best_times_response.html")]
struct ShortcutBestTimesFragment {
    session: Session,
    shortcut_best_times: Vec<db::ShortcutTime>,
}

#[derive(Template)]
#[template(path = "partials/shortcut_all_times_response.html")]
struct ShortcutAllTimesFragment {
    session: Session,
    shortcut_all_times: Vec<db::ShortcutTime>,
}

// -- Web: app state & helpers -------------------------------------------------

#[derive(Clone)]
struct AppState {
    pool: Arc<db::Pool>,
    oauth_client: BasicClient,
}

struct PageCtx {
    pub current_user: Option<String>,
    pub admin: bool,
    pub csrf_token: String,
}

impl FromRequestParts<AppState> for PageCtx {
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, StatusCode> {
        let session = TowerSession::from_request_parts(parts, state)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let auth_session = AuthSession::from_request_parts(parts, state)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let current_user = auth_session.user.as_ref().map(|u| u.uname.clone());
        let admin = auth_session.user.map(|u| u.admin).unwrap_or(false);
        let csrf_token = get_or_create_csrf_token(&session).await?;
        Ok(PageCtx { current_user, admin, csrf_token })
    }
}

async fn get_or_create_csrf_token(session: &TowerSession) -> Result<String, StatusCode> {
    if let Some(token) = session
        .get::<String>("csrf_token")
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    {
        return Ok(token);
    }
    let token = CsrfToken::new_random().secret().clone();
    session
        .insert("csrf_token", &token)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(token)
}

fn build_oauth_client(client_id: &str, redirect_uri: &str) -> anyhow::Result<BasicClient> {
    Ok(BasicClient::new(
        ClientId::new(client_id.to_string()),
        None,
        AuthUrl::new("https://id.lfs.net/oauth2/authorize".to_string())?,
        None,
    )
    .set_redirect_uri(RedirectUrl::new(redirect_uri.to_string())?))
}

fn group_metronome_rounds(results: Vec<db::MetronomeResult>) -> Vec<RoundResults> {
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

// -- Web: static assets -------------------------------------------------------

async fn logo() -> (StatusCode, [(&'static str, &'static str); 1], &'static [u8]) {
    (
        StatusCode::OK,
        [("content-type", "image/svg+xml")],
        include_bytes!("../logo.svg"),
    )
}

// -- Web: page handlers -------------------------------------------------------

async fn index(
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

async fn sessions(
    page: PageCtx,
    State(state): State<AppState>,
) -> Result<Html<String>, StatusCode> {
    let sessions = db::all_sessions(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let tmpl = SessionsTemplate { page, sessions };
    Ok(Html(tmpl.render().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?))
}

async fn session_detail(
    page: PageCtx,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Html<String>, StatusCode> {
    let session = db::get_session(&state.pool, id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let (metronome_standings, metronome_rounds, shortcut_best_times) = match &*session.mode {
        SessionMode::Metronome { .. } => {
            let standings = db::metronome_standings(&state.pool, session.id)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            let rounds = group_metronome_rounds(
                db::metronome_all_results(&state.pool, session.id)
                    .await
                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
            );
            (standings, rounds, vec![])
        }
        SessionMode::Shortcut => {
            let t = db::shortcut_best_times(&state.pool, session.id)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            (vec![], vec![], t)
        }
    };

    let tmpl = SessionDetailTemplate { page, session, metronome_standings, metronome_rounds, shortcut_best_times };
    Ok(Html(tmpl.render().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?))
}

#[derive(serde::Deserialize)]
struct StartSessionForm {
    csrf_token: String,
}

async fn session_start(
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

#[derive(serde::Deserialize)]
struct NewSessionForm {
    csrf_token: String,
    mode: String,
    track: String,
    #[serde(default)]
    layout: String,
    name: Option<String>,
    description: Option<String>,
    scheduled_at: Option<String>,
    #[serde(default = "default_rounds")]
    rounds: i64,
    #[serde(default = "default_target")]
    target: u64,
    #[serde(default = "default_max_scorers")]
    max_scorers: i64,
}

async fn session_new_get(page: PageCtx) -> Result<Html<String>, StatusCode> {
    if !page.admin {
        return Err(StatusCode::NOT_FOUND);
    }
    let tmpl = SessionNewTemplate { page, tracks: Track::ALL };
    Ok(Html(tmpl.render().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?))
}

async fn session_new_post(
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
        _ => return Err(StatusCode::BAD_REQUEST),
    };

    Ok(Redirect::to(&format!("/sessions/{id}")))
}

async fn session_edit_get(
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
struct EditSessionForm {
    csrf_token: String,
    track: String,
    #[serde(default)]
    layout: String,
    name: Option<String>,
    description: Option<String>,
    scheduled_at: Option<String>,
    writeup: Option<String>,
    rounds: Option<i64>,
    target: Option<u64>,
    max_scorers: Option<i64>,
}

async fn session_edit_post(
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

async fn session_standings(
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
    };

    Ok(Html(html))
}

async fn session_round(
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

async fn session_standings_best(
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

async fn session_standings_all(
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

async fn session_cancel(
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

// -- Web: auth handlers -------------------------------------------------------

async fn login(
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
struct AuthzResp {
    code: String,
    state: String,
}

async fn callback(
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

async fn logout(mut auth_session: AuthSession) -> impl IntoResponse {
    let _ = auth_session.logout().await.unwrap();
    Redirect::to("/")
}

// -- Runner -------------------------------------------------------------------

async fn run_loop(
    pool: db::Pool,
    addr: SocketAddr,
    password: Option<String>,
    listen: SocketAddr,
) -> anyhow::Result<()> {
    let (insim, insim_handle) = insim::tcp(addr)
        .isi_admin_password(password)
        .isi_iname("carnage".to_owned())
        .isi_prefix('!')
        .isi_flag_mso_cols(true)
        .spawn(100)
        .await?;

    tracing::info!("Connected to InSim");

    let (presence, presence_handle) = presence::spawn(insim.clone(), 32);
    let (game, game_handle) = game::spawn(insim.clone(), 32);
    let user_sync_handle = db::spawn_user_sync(&presence, pool.clone());

    insim.send(TinyType::Ncn.with_request_id(1)).await?;
    insim.send(TinyType::Npl.with_request_id(2)).await?;
    insim.send(TinyType::Sst.with_request_id(3)).await?;

    for &cmd in &["/select no", "/vote no", "/autokick no"] {
        insim.send_command(cmd).await?;
    }

    let ctx = GameCtx {
        pool: pool.clone(),
        insim: insim.clone(),
        presence,
        game,
    };

    let reconcile = async {
        let mut current_session_id: Option<i64> = None;
        let mut current_task: Option<tokio::task::JoinHandle<Result<(), kitcar::scenes::SceneError>>> = None;

        loop {
            if let Ok(Some(session)) = db::next_scheduled_session(&pool).await {
                tracing::info!("Auto-activating scheduled session #{}", session.id);
                let _ = db::switch_session(&pool, session.id).await;
            }

            let desired = db::active_session(&pool).await;

            match (&current_task, desired) {
                (_, Err(e)) => {
                    tracing::warn!("Failed to poll active session: {e}");
                },

                (None, Ok(None)) => {},

                (None, Ok(Some(session))) => {
                    tracing::info!(
                        "Starting session #{} ({:?} on {}/{})",
                        session.id, session.mode, session.track, session.layout
                    );
                    current_session_id = Some(session.id);
                    let ctx_ref = &ctx;
                    current_task = Some(tokio::spawn({
                        let session = session.clone();
                        let pool = ctx_ref.pool.clone();
                        let insim = ctx_ref.insim.clone();
                        let presence = ctx_ref.presence.clone();
                        let game = ctx_ref.game.clone();
                        async move {
                            let ctx = GameCtx { pool, insim, presence, game };
                            match session.mode {
                                Json(SessionMode::Metronome { .. }) => {
                                    execute::<games::metronome::MetronomeGame>(&session, &ctx).await
                                },
                                Json(SessionMode::Shortcut) => {
                                    execute::<games::shortcut::ShortcutGame>(&session, &ctx).await
                                },
                            }
                        }
                    }));
                },

                (Some(task), Ok(Some(session)))
                    if current_session_id == Some(session.id) && !task.is_finished() => {},

                (Some(_), Ok(Some(session)))
                    if current_session_id == Some(session.id) =>
                {
                    let task = current_task.take().unwrap();
                    match task.await {
                        Ok(Ok(())) => {
                            tracing::info!("Session #{} completed", session.id);
                        },
                        Ok(Err(e)) => {
                            tracing::error!(
                                "Session #{} failed: {e:?} (leaving ACTIVE for crash recovery)",
                                session.id
                            );
                        },
                        Err(e) => {
                            tracing::error!(
                                "Session #{} join error: {e} (leaving ACTIVE for crash recovery)",
                                session.id
                            );
                        },
                    }
                    current_session_id = None;
                },

                (Some(_), Ok(_)) => {
                    tracing::info!("Desired session changed, aborting current task");
                    if let Some(task) = current_task.take() {
                        task.abort();
                    }
                    current_session_id = None;
                },
            }

            tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        }
    };

    // Set up web server
    let client_id = std::env::var("LFS_CLIENT_ID")?;
    let client_secret = std::env::var("LFS_CLIENT_SECRET")?;
    let redirect_uri = std::env::var("LFS_REDIRECT_URI")?;

    let oauth_client = build_oauth_client(&client_id, &redirect_uri)?;
    let backend = Backend::new(pool.clone(), client_id, client_secret, redirect_uri);

    let session_store = SqliteStore::new(pool.clone());
    session_store.migrate().await?;

    let key = Key::from(
        std::env::var("SESSION_KEY")
            .unwrap_or_else(|_| "a".repeat(64))
            .as_bytes(),
    );

    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_same_site(SameSite::Lax)
        .with_private(key);

    let auth_layer = AuthManagerLayerBuilder::new(backend, session_layer).build();

    let app_state = AppState {
        pool: Arc::new(pool.clone()),
        oauth_client,
    };

    let app = Router::new()
        .route("/logo.svg", get(logo))
        .route("/", get(index))
        .route("/sessions", get(sessions))
        .route("/sessions/new", get(session_new_get).post(session_new_post))
        .route("/sessions/{id}", get(session_detail))
        .route("/sessions/{id}/edit", get(session_edit_get).post(session_edit_post))
        .route("/sessions/{id}/standings", get(session_standings))
        .route("/sessions/{id}/rounds/{round}", get(session_round))
        .route("/sessions/{id}/standings/best", get(session_standings_best))
        .route("/sessions/{id}/standings/all", get(session_standings_all))
        .route("/sessions/{id}/start", post(session_start))
        .route("/sessions/{id}/cancel", post(session_cancel))
        .route("/login", get(login))
        .route("/auth/callback", get(callback))
        .route("/logout", get(logout))
        .layer(auth_layer)
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind(listen).await?;
    tracing::info!("Web listening on {listen}");

    tokio::select! {
        _ = reconcile => {},
        result = axum::serve(listener, app) => {
            if let Err(e) = result {
                tracing::error!("Web server error: {e}");
            }
        },
        res = insim_handle => {
            match res {
                Ok(Ok(())) => tracing::info!("InSim background task exited"),
                Ok(Err(e)) => tracing::error!("InSim background task failed: {e:?}"),
                Err(e) => tracing::error!("InSim background task join failed: {e}"),
            }
        },
        res = presence_handle => {
            match res {
                Ok(Ok(())) => tracing::info!("Presence background task exited"),
                Ok(Err(e)) => tracing::error!("Presence background task failed: {e}"),
                Err(e) => tracing::error!("Presence background task join failed: {e}"),
            }
        },
        res = game_handle => {
            match res {
                Ok(Ok(())) => tracing::info!("Game background task exited"),
                Ok(Err(e)) => tracing::error!("Game background task failed: {e}"),
                Err(e) => tracing::error!("Game background task join failed: {e}"),
            }
        },
        res = user_sync_handle => {
            match res {
                Ok(Ok(())) => tracing::info!("User sync background task exited"),
                Ok(Err(e)) => tracing::error!("User sync background task failed: {e}"),
                Err(e) => tracing::error!("User sync background task join failed: {e}"),
            }
        },
    }

    Ok(())
}

// -- Entry point --------------------------------------------------------------

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive(tracing_subscriber::filter::LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .init();

    let args = Args::parse();
    let pool = db::connect(&args.db).await?;

    match args.command {
        Command::Add { mode } => match mode {
            AddMode::Metronome {
                track,
                layout,
                rounds,
                target,
                max_scorers,
                lobby_duration_secs,
                name,
                description,
                scheduled_at,
            } => {
                let target_ms = (target * 1000) as i64;
                let id = db::create_metronome_session(
                    &pool,
                    &db::CreateMetronomeParams {
                        track,
                        layout,
                        rounds,
                        target_ms,
                        max_scorers,
                        lobby_duration_secs: lobby_duration_secs as i64,
                        name,
                        description,
                        scheduled_at,
                    },
                )
                .await?;
                println!("Created metronome session #{id}");
            },
            AddMode::Shortcut { track, layout, name, description, scheduled_at } => {
                let id = db::create_shortcut_session(
                    &pool,
                    &db::CreateShortcutParams { track, layout, name, description, scheduled_at },
                )
                .await?;
                println!("Created shortcut session #{id}");
            },
        },

        Command::List => {
            let sessions = db::all_sessions(&pool).await?;
            if sessions.is_empty() {
                println!("No sessions.");
            } else {
                for s in sessions {
                    let label = s.name.as_deref().unwrap_or("");
                    println!(
                        "#{} {:?} {:?} {}/{} {} ({})",
                        s.id, s.mode, s.status, s.track, s.layout, label, s.created_at
                    );
                }
            }
        },

        Command::Activate { id } => {
            match db::pending_session(&pool, id).await? {
                Some(_) => {
                    db::activate_session(&pool, id).await?;
                    println!("Activated session #{id}");
                },
                None => {
                    eprintln!("Session #{id} not found or not in PENDING status.");
                    std::process::exit(1);
                },
            }
        },

        Command::Writeup { id, text } => {
            db::update_session_writeup(&pool, id, &text).await?;
            println!("Updated write-up for session #{id}");
        },

        Command::Run { addr, password, listen } => {
            run_loop(pool, addr, password, listen).await?;
        },
    }

    Ok(())
}
