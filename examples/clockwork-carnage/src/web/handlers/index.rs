use askama::Template;
use axum::{
    extract::State,
    http::StatusCode,
    response::Html,
};

use crate::db::{self, Session, SessionMode};
use crate::web::state::{AppState, PageCtx};

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    pub page: PageCtx,
    pub active: Option<Session>,
    pub upcoming: Vec<Session>,
}

pub async fn index(
    page: PageCtx,
    State(state): State<AppState>,
) -> Result<Html<String>, StatusCode> {
    let active = db::active_session(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let upcoming = db::upcoming_sessions(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let tmpl = IndexTemplate { page, active, upcoming };
    Ok(Html(tmpl.render().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?))
}
