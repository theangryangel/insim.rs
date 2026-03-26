use askama::Template;
use axum::{extract::State, http::StatusCode, response::Html};
use insim::identifiers::ConnectionId;

use super::internal_error;
use crate::{
    db::{self, Event, EventMode, User},
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

pub struct PresenceRow {
    pub uname: String,
    pub pname: String,
    pub twitch_username: Option<String>,
    pub youtube_username: Option<String>,
}

impl From<User> for PresenceRow {
    fn from(value: User) -> Self {
        Self {
            uname: value.uname,
            pname: value.pname,
            twitch_username: value.twitch_username,
            youtube_username: value.youtube_username,
        }
    }
}

#[derive(Template)]
#[template(path = "partials/presence.html")]
pub struct PresenceTemplate {
    pub connections: Vec<PresenceRow>,
}

pub async fn presence_partial(State(state): State<AppState>) -> Result<Html<String>, StatusCode> {
    let raw_connections = match &state.presence {
        Some(presence) => presence.connections().await.unwrap_or_default(),
        None => vec![],
    };

    let unames: Vec<&str> = raw_connections
        .iter()
        .filter_map(|c| {
            if c.ucid == ConnectionId::LOCAL {
                None
            } else {
                Some(c.uname.as_ref())
            }
        })
        .collect();
    let users = db::users_for_unames(&state.pool, &unames)
        .await
        .map_err(internal_error)?;

    let connections: Vec<PresenceRow> = users.into_iter().map(|u| u.into()).collect();

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
