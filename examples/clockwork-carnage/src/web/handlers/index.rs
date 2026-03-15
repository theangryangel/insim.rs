use askama::Template;
use axum::{
    extract::State,
    http::StatusCode,
    response::Html,
};

use crate::db::{self, Event, EventMode};
use crate::web::state::{AppState, PageCtx};

use super::internal_error;

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    pub page: PageCtx,
    pub active: Option<Event>,
    pub upcoming: Vec<Event>,
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
    let tmpl = IndexTemplate { page, active, upcoming };
    Ok(Html(tmpl.render().map_err(internal_error)?))
}
