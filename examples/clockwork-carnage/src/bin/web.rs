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
use clockwork_carnage::db::{self, Challenge, ChallengeTime, Event, EventRoundResult, EventStanding};

#[derive(Debug, Parser)]
struct Args {
    #[arg(long, default_value = "clockwork-carnage.db")]
    db: String,

    #[arg(long, default_value = "127.0.0.1:3000")]
    listen: String,
}

type AppState = Arc<db::Pool>;

// -- Filters ------------------------------------------------------------------

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
    active_event: Option<Event>,
    active_challenge: Option<Challenge>,
    events: Vec<Event>,
    challenges: Vec<Challenge>,
    active_count: usize,
}

#[derive(Template)]
#[template(path = "events.html")]
struct EventsTemplate {
    events: Vec<Event>,
}

#[derive(Template)]
#[template(path = "event_detail.html")]
struct EventDetailTemplate {
    event: Event,
    standings: Vec<EventStanding>,
    rounds: Vec<Vec<EventRoundResult>>,
}

#[derive(Template)]
#[template(path = "challenges.html")]
struct ChallengesTemplate {
    challenges: Vec<Challenge>,
}

#[derive(Template)]
#[template(path = "challenge_detail.html")]
struct ChallengeDetailTemplate {
    challenge: Challenge,
    times: Vec<ChallengeTime>,
}

// -- Handlers -----------------------------------------------------------------

async fn logo() -> (StatusCode, [(&'static str, &'static str); 1], &'static [u8]) {
    (
        StatusCode::OK,
        [("content-type", "image/png")],
        include_bytes!("../../logo.png"),
    )
}

async fn index(State(pool): State<AppState>) -> Result<Html<String>, StatusCode> {
    let active_event = db::any_active_event(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let active_challenge = db::any_active_challenge(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let events = db::all_events(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let challenges = db::all_challenges(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let active_count = events.iter().filter(|e| e.ended_at.is_none()).count()
        + challenges.iter().filter(|c| c.ended_at.is_none()).count();

    let tmpl = IndexTemplate {
        active_event,
        active_challenge,
        events,
        challenges,
        active_count,
    };
    Ok(Html(tmpl.render().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?))
}

async fn events(State(pool): State<AppState>) -> Result<Html<String>, StatusCode> {
    let events = db::all_events(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let tmpl = EventsTemplate { events };
    Ok(Html(tmpl.render().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?))
}

async fn event_detail(
    State(pool): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Html<String>, StatusCode> {
    let event = db::get_event(&pool, id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let standings = db::event_standings(&pool, id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let num_rounds = if event.ended_at.is_some() {
        event.rounds
    } else {
        event.current_round
    };

    let mut rounds = Vec::new();
    for r in 1..=num_rounds {
        let results = db::event_round_results(&pool, id, r)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        rounds.push(results);
    }

    let tmpl = EventDetailTemplate {
        event,
        standings,
        rounds,
    };
    Ok(Html(tmpl.render().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?))
}

async fn challenges(State(pool): State<AppState>) -> Result<Html<String>, StatusCode> {
    let challenges = db::all_challenges(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let tmpl = ChallengesTemplate { challenges };
    Ok(Html(tmpl.render().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?))
}

async fn challenge_detail(
    State(pool): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Html<String>, StatusCode> {
    let challenge = db::get_challenge(&pool, id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let times = db::challenge_best_times(&pool, id, 100)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let tmpl = ChallengeDetailTemplate { challenge, times };
    Ok(Html(tmpl.render().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?))
}

// -- Main ---------------------------------------------------------------------

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let args = Args::parse();
    let pool = db::connect(&args.db).await?;
    let state: AppState = Arc::new(pool);

    let app = Router::new()
        .route("/logo.png", get(logo))
        .route("/", get(index))
        .route("/events", get(events))
        .route("/events/{id}", get(event_detail))
        .route("/challenges", get(challenges))
        .route("/challenges/{id}", get(challenge_detail))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&args.listen).await?;
    tracing::info!("Listening on {}", args.listen);
    axum::serve(listener, app).await?;

    Ok(())
}
