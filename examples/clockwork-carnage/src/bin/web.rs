//! Clockwork Carnage — web dashboard with LFS OAuth login.

use std::sync::Arc;

use askama::Template;
use axum::{
    Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect},
    routing::get,
};
use axum_login::AuthManagerLayerBuilder;
use clap::Parser;
use clockwork_carnage::{
    db::{self, Session, SessionMode, SessionStatus},
    web::{AuthSession, Backend, OAuthCredentials},
};
use oauth2::{AuthUrl, ClientId, CsrfToken, RedirectUrl, Scope, basic::BasicClient};
use tower_sessions::{MemoryStore, SessionManagerLayer, cookie::SameSite};
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
    active: Option<Session>,
    upcoming: Vec<Session>,
    current_user: Option<String>,
}

#[derive(Template)]
#[template(path = "about.html")]
struct AboutTemplate {
    current_user: Option<String>,
}

#[derive(Template)]
#[template(path = "sessions.html")]
struct SessionsTemplate {
    sessions: Vec<Session>,
    current_user: Option<String>,
}

#[derive(Template)]
#[template(path = "session_detail.html")]
struct SessionDetailTemplate {
    session: Session,
    current_user: Option<String>,
}

// -- Helpers ------------------------------------------------------------------

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
    auth_session: AuthSession,
    State(state): State<AppState>,
) -> Result<Html<String>, StatusCode> {
    let current_user = auth_session.user.map(|u| u.uname.clone());
    let active = db::active_session(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let upcoming = db::upcoming_sessions(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let tmpl = IndexTemplate { active, upcoming, current_user };
    Ok(Html(
        tmpl.render().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
    ))
}

async fn about(auth_session: AuthSession) -> Result<Html<String>, StatusCode> {
    let current_user = auth_session.user.map(|u| u.uname.clone());
    let tmpl = AboutTemplate { current_user };
    Ok(Html(
        tmpl.render().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
    ))
}

async fn sessions(
    auth_session: AuthSession,
    State(state): State<AppState>,
) -> Result<Html<String>, StatusCode> {
    let current_user = auth_session.user.map(|u| u.uname.clone());
    let sessions = db::all_sessions(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let tmpl = SessionsTemplate { sessions, current_user };
    Ok(Html(
        tmpl.render().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
    ))
}

async fn session_detail(
    auth_session: AuthSession,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Html<String>, StatusCode> {
    let current_user = auth_session.user.map(|u| u.uname.clone());
    let session = db::get_session(&state.pool, id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    let tmpl = SessionDetailTemplate { session, current_user };
    Ok(Html(
        tmpl.render().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
    ))
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

    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_same_site(SameSite::Lax);

    let auth_layer = AuthManagerLayerBuilder::new(backend, session_layer).build();

    let state = AppState {
        pool: Arc::new(pool),
        oauth_client,
    };

    let app = Router::new()
        .route("/logo.svg", get(logo))
        .route("/", get(index))
        .route("/about", get(about))
        .route("/sessions", get(sessions))
        .route("/sessions/{id}", get(session_detail))
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
