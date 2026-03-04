//! Clockwork Carnage — Read-only web dashboard.

use std::sync::Arc;

use askama::Template;
use axum::{
    Router,
    extract::{Path, State},
    http::StatusCode,
    response::Html,
    routing::get,
};
use clap::Parser;
use clockwork_carnage::db::{
    self, Session, SessionMode, SessionStatus,
};

#[derive(Debug, Parser)]
struct Args {
    #[arg(long, default_value = "clockwork-carnage.db")]
    db: String,

    #[arg(long, default_value = "127.0.0.1:3000")]
    listen: String,
}

type AppState = Arc<db::Pool>;

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
}

#[derive(Template)]
#[template(path = "about.html")]
struct AboutTemplate {
}

#[derive(Template)]
#[template(path = "sessions.html")]
struct SessionsTemplate {
    sessions: Vec<Session>,
}

#[derive(Template)]
#[template(path = "session_detail.html")]
struct SessionDetailTemplate {
    session: Session,
}

async fn logo() -> (StatusCode, [(&'static str, &'static str); 1], &'static [u8]) {
    (
        StatusCode::OK,
        [("content-type", "image/svg+xml")],
        include_bytes!("../../logo.svg"),
    )
}

async fn index(State(pool): State<AppState>) -> Result<Html<String>, StatusCode> {
    let active = db::active_session(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let upcoming = db::upcoming_sessions(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let tmpl = IndexTemplate {
        active, upcoming 
    };
    Ok(Html(tmpl.render().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?))
}

async fn about() -> Result<Html<String>, StatusCode> {
    let tmpl = AboutTemplate {};
    Ok(Html(tmpl.render().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?))
}

async fn sessions(State(pool): State<AppState>) -> Result<Html<String>, StatusCode> {
    let sessions = db::all_sessions(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let tmpl = SessionsTemplate {
        sessions
    };
    Ok(Html(tmpl.render().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?))
}

async fn session_detail(
    State(pool): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Html<String>, StatusCode> {
    let session = db::get_session(&pool, id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let tmpl = SessionDetailTemplate {
        session,
    };
    Ok(Html(tmpl.render().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let args = Args::parse();
    let pool = db::connect(&args.db).await?;
    let state: AppState = Arc::new(pool);

    let app = Router::new()
        .route("/logo.svg", get(logo))
        .route("/", get(index))
        .route("/about", get(about))
        .route("/sessions", get(sessions))
        .route("/sessions/{id}", get(session_detail))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&args.listen).await?;
    tracing::info!("Listening on {}", args.listen);
    axum::serve(listener, app).await?;

    Ok(())
}
