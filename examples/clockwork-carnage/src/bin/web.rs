//! Clockwork Carnage — web dashboard with LFS OAuth login.

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
use clap::Parser;
use clockwork_carnage::{
    db::{self, Session, SessionMode, SessionStatus},
    web::{AuthSession, Backend, OAuthCredentials},
};
use insim::core::track::Track;
use oauth2::{AuthUrl, ClientId, CsrfToken, RedirectUrl, Scope, basic::BasicClient};
use tower_sessions::{SessionManagerLayer, cookie::SameSite, cookie::Key};
use tower_sessions_sqlx_store::SqliteStore;
use tower_sessions::Session as TowerSession;

#[derive(Debug, Parser)]
struct Args {
    #[arg(long, default_value = "clockwork-carnage.db")]
    db: String,

    #[arg(long, default_value = "127.0.0.1:3000")]
    listen: String,
}

#[derive(Clone)]
struct AppState {
    pool: Arc<db::Pool>,
    oauth_client: BasicClient,
}

mod filters {
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
}

// -- Templates ----------------------------------------------------------------

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

#[derive(Template)]
#[template(path = "session_detail.html")]
struct SessionDetailTemplate {
    page: PageCtx,
    session: Session,
    metronome_standings: Vec<db::MetronomeStanding>,
    shortcut_best_times: Vec<db::ShortcutTime>,
}

#[derive(Template)]
#[template(path = "session_new.html")]
struct SessionNewTemplate {
    page: PageCtx,
    tracks: &'static [Track],
}

// -- Helpers ------------------------------------------------------------------

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

fn build_oauth_client(
    client_id: &str,
    redirect_uri: &str,
) -> anyhow::Result<BasicClient> {
    Ok(BasicClient::new(
        ClientId::new(client_id.to_string()),
        None,
        AuthUrl::new("https://id.lfs.net/oauth2/authorize".to_string())?,
        None,
    )
    .set_redirect_uri(RedirectUrl::new(redirect_uri.to_string())?))
}

// -- Static assets ------------------------------------------------------------

async fn logo() -> (StatusCode, [(&'static str, &'static str); 1], &'static [u8]) {
    (
        StatusCode::OK,
        [("content-type", "image/svg+xml")],
        include_bytes!("../../logo.svg"),
    )
}

// -- Page handlers ------------------------------------------------------------

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
    Ok(Html(
        tmpl.render().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
    ))
}

async fn sessions(
    page: PageCtx,
    State(state): State<AppState>,
) -> Result<Html<String>, StatusCode> {
    let sessions = db::all_sessions(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let tmpl = SessionsTemplate { page, sessions };
    Ok(Html(
        tmpl.render().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
    ))
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

    let (metronome_standings, shortcut_best_times) = match &*session.mode {
        SessionMode::Metronome { .. } => {
            let s = db::metronome_standings(&state.pool, session.id)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            (s, vec![])
        }
        SessionMode::Shortcut => {
            let t = db::shortcut_best_times(&state.pool, session.id, 20)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            (vec![], t)
        }
    };

    let tmpl = SessionDetailTemplate { page, session, metronome_standings, shortcut_best_times };
    Ok(Html(
        tmpl.render().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
    ))
}

#[derive(serde::Deserialize)]
struct StartSessionForm {
    csrf_token: String,
}

async fn session_start(
    page: PageCtx,
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Form(form): Form<StartSessionForm>,
) -> Result<Redirect, StatusCode> {
    if !page.admin {
        return Err(StatusCode::NOT_FOUND);
    }
    if form.csrf_token != page.csrf_token {
        return Err(StatusCode::FORBIDDEN);
    }
    db::switch_session(&state.pool, id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Redirect::to("/"))
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

async fn session_new_get(
    page: PageCtx,
) -> Result<Html<String>, StatusCode> {
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

// -- Auth handlers ------------------------------------------------------------

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
    let expected: Option<String> = session
        .remove("oauth_csrf_state")
        .await
        .unwrap();
    if expected.as_deref() != Some(&params.state) {
        return StatusCode::BAD_REQUEST.into_response();
    }
    let creds = OAuthCredentials {
        code: params.code,
        state: params.state,
    };
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

// -- Main ---------------------------------------------------------------------

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let args = Args::parse();
    let pool = db::connect(&args.db).await?;

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

    let state = AppState {
        pool: Arc::new(pool),
        oauth_client,
    };

    let app = Router::new()
        .route("/logo.svg", get(logo))
        .route("/", get(index))
        .route("/sessions", get(sessions))
        .route("/sessions/new", get(session_new_get).post(session_new_post))
        .route("/sessions/{id}", get(session_detail))
        .route("/sessions/{id}/start", post(session_start))
        .route("/login", get(login))
        .route("/auth/callback", get(callback))
        .route("/logout", get(logout))
        .layer(auth_layer)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&args.listen).await?;
    tracing::info!("Listening on {}", args.listen);
    axum::serve(listener, app).await?;

    Ok(())
}
