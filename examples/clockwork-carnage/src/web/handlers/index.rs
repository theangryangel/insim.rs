use askama::Template;
use axum::{extract::State, http::StatusCode, response::Html};
use std::collections::HashMap;

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

pub struct PresenceRow {
    pub uname: String,
    pub pname: String,
    pub twitch_username: Option<String>,
    pub youtube_username: Option<String>,
}

#[derive(Template)]
#[template(path = "partials/presence.html")]
pub struct PresenceTemplate {
    pub connections: Vec<PresenceRow>,
}

pub async fn presence_partial(
    State(state): State<AppState>,
) -> Result<Html<String>, StatusCode> {
    let raw_connections = match &state.presence {
        Some(presence) => presence.connections().await.unwrap_or_default(),
        None => vec![],
    };

    let unames: Vec<String> = raw_connections.iter().map(|c| c.uname.clone()).collect();
    let users = db::users_for_unames(&state.pool, &unames)
        .await
        .map_err(internal_error)?;
    let user_map: HashMap<String, (Option<String>, Option<String>)> = users
        .into_iter()
        .map(|u| (u.uname, (u.twitch_username, u.youtube_username)))
        .collect();

    let connections = raw_connections
        .into_iter()
        .map(|c| {
            let (twitch_username, youtube_username) = user_map
                .get(&c.uname)
                .map(|(t, y)| (t.clone(), y.clone()))
                .unwrap_or((None, None));
            PresenceRow {
                uname: c.uname,
                pname: c.pname,
                twitch_username,
                youtube_username,
            }
        })
        .collect();

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
