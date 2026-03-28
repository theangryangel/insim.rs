use askama::Template;
use axum::{extract::State, http::StatusCode, response::Html};

use super::internal_error;
use crate::web::filters;
use crate::{
    db::{self, XpLeaderboardRow},
    web::state::{AppState, PageCtx},
};

#[derive(Template)]
#[template(path = "leaderboard.html")]
pub struct LeaderboardTemplate {
    pub page: PageCtx,
    pub rows: Vec<XpLeaderboardRow>,
}

pub async fn leaderboard(
    page: PageCtx,
    State(state): State<AppState>,
) -> Result<Html<String>, StatusCode> {
    let rows = db::get_top_xp(&state.pool, 10)
        .await
        .map_err(internal_error)?;
    let tmpl = LeaderboardTemplate { page, rows };
    Ok(Html(tmpl.render().map_err(internal_error)?))
}
