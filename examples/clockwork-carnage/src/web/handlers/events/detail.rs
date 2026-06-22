use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Html,
};

use super::internal_error;
use crate::{
    db::{self, Event, EventMode},
    web::{
        AuthSession,
        state::{AppState, User},
        views,
    },
};

pub enum EventResults {
    Metronome {
        standings: Vec<db::MetronomeStanding>,
        target_ms: i64,
    },
    Shortcut {
        best: Vec<db::ShortcutTime>,
        all: Vec<db::ShortcutTime>,
    },
    Bomb {
        best: Vec<db::BombRun>,
        all: Vec<db::BombRun>,
    },
}

/// Load the leaderboard/results for an event. Shared by the full page render
/// and the standalone fragment route.
pub async fn load_event_results(
    pool: &db::Pool,
    event: &Event,
) -> Result<EventResults, StatusCode> {
    Ok(match &*event.mode {
        EventMode::Metronome { target_ms } => {
            let standings = db::metronome_standings(pool, event.id)
                .await
                .map_err(internal_error)?;
            EventResults::Metronome {
                standings,
                target_ms: *target_ms,
            }
        },
        EventMode::Shortcut => {
            let best = db::shortcut_best_times(pool, event.id)
                .await
                .map_err(internal_error)?;
            let all = db::shortcut_all_times(pool, event.id)
                .await
                .map_err(internal_error)?;
            EventResults::Shortcut { best, all }
        },
        EventMode::Bomb { .. } => {
            let best = db::bomb_best_runs(pool, event.id)
                .await
                .map_err(internal_error)?;
            let all = db::bomb_all_runs(pool, event.id)
                .await
                .map_err(internal_error)?;
            EventResults::Bomb { best, all }
        },
    })
}

pub async fn event_detail(
    auth: AuthSession,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Html<String>, StatusCode> {
    let current_user = User::from(&auth);
    let event = db::get_event(&state.pool, id)
        .await
        .map_err(internal_error)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let results = load_event_results(&state.pool, &event).await?;

    Ok(Html(
        crate::web::views::event_detail(&current_user, &event, &results).into_string(),
    ))
}

/// Standalone `#event-results` fragment — same Maud function as the page embeds.
/// This is the seam an SSE push (or poll) would render into.
pub async fn event_results_fragment(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Html<String>, StatusCode> {
    let event = db::get_event(&state.pool, id)
        .await
        .map_err(internal_error)?
        .ok_or(StatusCode::NOT_FOUND)?;
    let results = load_event_results(&state.pool, &event).await?;
    Ok(Html(views::event_results(&results).into_string()))
}
