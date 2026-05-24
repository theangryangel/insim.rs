use askama::Template;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Html,
};

use super::internal_error;
use crate::{
    db::{self, Event, EventMode, EventStatus},
    web::{
        AuthSession, filters,
        state::{AppState, User},
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

#[derive(Template)]
#[template(path = "event_detail.html")]
pub struct EventDetailTemplate {
    pub current_user: User,
    pub event: Event,
    pub results: EventResults,
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

    let results = match &*event.mode {
        EventMode::Metronome { target_ms } => {
            let standings = db::metronome_standings(&state.pool, event.id)
                .await
                .map_err(internal_error)?;
            EventResults::Metronome {
                standings,
                target_ms: *target_ms,
            }
        },
        EventMode::Shortcut => {
            let best = db::shortcut_best_times(&state.pool, event.id)
                .await
                .map_err(internal_error)?;
            let all = db::shortcut_all_times(&state.pool, event.id)
                .await
                .map_err(internal_error)?;
            EventResults::Shortcut { best, all }
        },
        EventMode::Bomb { .. } => {
            let best = db::bomb_best_runs(&state.pool, event.id)
                .await
                .map_err(internal_error)?;
            let all = db::bomb_all_runs(&state.pool, event.id)
                .await
                .map_err(internal_error)?;
            EventResults::Bomb { best, all }
        },
    };

    Ok(Html(
        EventDetailTemplate {
            current_user,
            event,
            results,
        }
        .render()
        .map_err(internal_error)?,
    ))
}
