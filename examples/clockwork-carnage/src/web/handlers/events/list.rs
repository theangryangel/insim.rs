use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Html,
};

use super::internal_error;
use crate::{
    db,
    web::{
        AuthSession,
        state::{AppState, User},
    },
};

const PER_PAGE: i64 = 20;

#[derive(serde::Deserialize, Default)]
pub struct EventsQuery {
    pub status: Option<String>,
    pub mode: Option<String>,
    pub era: Option<i64>,
    pub page: Option<i64>,
}

pub async fn events(
    auth: AuthSession,
    Query(query): Query<EventsQuery>,
    State(state): State<AppState>,
) -> Result<Html<String>, StatusCode> {
    let current_user = User::from(&auth);

    let status = query.status.as_deref().filter(|s| !s.is_empty());
    let mode_filter = query.mode.as_deref().filter(|m| !m.is_empty());
    let era_filter = query.era;
    let page = query.page.unwrap_or(1).max(1);
    let offset = (page - 1) * PER_PAGE;

    let (events, total, eras) = tokio::try_join!(
        db::filtered_events(
            &state.pool,
            status,
            mode_filter,
            era_filter,
            offset,
            PER_PAGE
        ),
        db::count_filtered_events(&state.pool, status, mode_filter, era_filter),
        db::all_eras(&state.pool),
    )
    .map_err(internal_error)?;

    let total_pages = ((total + PER_PAGE - 1) / PER_PAGE).max(1);

    let filters = crate::web::views::Filters {
        status: status.unwrap_or(""),
        mode: mode_filter.unwrap_or(""),
        era: era_filter,
    };
    Ok(Html(
        crate::web::views::events(&current_user, &events, &eras, &filters, page, total_pages)
            .into_string(),
    ))
}
