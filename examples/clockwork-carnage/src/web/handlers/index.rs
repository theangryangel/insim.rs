use askama::Template;
use axum::{extract::State, http::StatusCode, response::Html};
use kitcar::presence::ConnectionInfo;

use super::internal_error;
use crate::{
    db::{self, Event, EventMode},
    web::{
        filters,
        state::{AppState, PageCtx},
    },
};

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    pub page: PageCtx,
    pub active: Option<Event>,
    pub upcoming: Vec<Event>,
}

#[derive(Template)]
#[template(path = "partials/presence.html")]
pub struct PresenceTemplate {
    pub connections: Vec<ConnectionInfo>,
}

pub async fn presence_partial(
    State(state): State<AppState>,
) -> Result<Html<String>, StatusCode> {
    let connections = match &state.presence {
        Some(presence) => presence.connections().await.unwrap_or_default(),
        None => vec![],
    };
    let tmpl = PresenceTemplate { connections };
    Ok(Html(tmpl.render().map_err(internal_error)?))
}

pub async fn index(
    page: PageCtx,
    State(state): State<AppState>,
) -> Result<Html<String>, StatusCode> {
    let active = db::active_event(&state.pool)
        .await
        .map_err(internal_error)?;
    let upcoming = db::upcoming_events(&state.pool)
        .await
        .map_err(internal_error)?;
    let tmpl = IndexTemplate {
        page,
        active,
        upcoming,
    };
    Ok(Html(tmpl.render().map_err(internal_error)?))
}
