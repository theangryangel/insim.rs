use askama::Template;
use axum::{extract::State, http::StatusCode, response::Html};

use super::internal_error;
use crate::{
    db::{self, Event, EventStatus},
    web::{
        AuthSession, filters,
        state::{AppState, User},
    },
};

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    pub current_user: User,
    pub events: Vec<Event>,
}

pub async fn index(
    auth: AuthSession,
    State(state): State<AppState>,
) -> Result<Html<String>, StatusCode> {
    let current_user = User::from(&auth);
    let mut events = Vec::new();
    if let Some(e) = db::ongoing_event(&state.pool)
        .await
        .map_err(internal_error)?
    {
        events.push(e);
    }
    events.extend(
        db::upcoming_events(&state.pool)
            .await
            .map_err(internal_error)?,
    );
    Ok(Html(
        IndexTemplate {
            current_user,
            events,
        }
        .render()
        .map_err(internal_error)?,
    ))
}
