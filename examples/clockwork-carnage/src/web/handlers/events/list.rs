use askama::Template;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Html,
};

use super::internal_error;
use crate::{
    db::{self, Era, Event, EventStatus},
    web::{
        AuthSession, filters,
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

#[derive(Template)]
#[template(path = "events.html")]
pub struct EventsTemplate {
    pub current_user: User,
    pub events: Vec<Event>,
    pub eras: Vec<Era>,
    pub filter_status: String,
    pub filter_mode: String,
    pub filter_era: Option<i64>,
    pub page: i64,
    pub total_pages: i64,
}

impl EventsTemplate {
    fn base_params(&self) -> Vec<String> {
        let mut params = vec![];
        if !self.filter_status.is_empty() {
            params.push(format!("status={}", self.filter_status));
        }
        if !self.filter_mode.is_empty() {
            params.push(format!("mode={}", self.filter_mode));
        }
        if let Some(era) = self.filter_era {
            params.push(format!("era={era}"));
        }
        params
    }

    fn build_url(params: Vec<String>) -> String {
        if params.is_empty() {
            "/events".to_string()
        } else {
            format!("/events?{}", params.join("&"))
        }
    }

    pub fn status_url(&self, s: &str) -> String {
        let mut params = self.base_params();
        params.retain(|p| !p.starts_with("status="));
        if !s.is_empty() {
            params.insert(0, format!("status={s}"));
        }
        Self::build_url(params)
    }

    pub fn mode_url(&self, m: &str) -> String {
        let mut params = self.base_params();
        params.retain(|p| !p.starts_with("mode="));
        if !m.is_empty() {
            params.push(format!("mode={m}"));
        }
        Self::build_url(params)
    }

    pub fn era_url(&self, era_id: Option<i64>) -> String {
        let mut params = self.base_params();
        params.retain(|p| !p.starts_with("era="));
        if let Some(id) = era_id {
            params.push(format!("era={id}"));
        }
        Self::build_url(params)
    }

    pub fn page_url(&self, p: i64) -> String {
        let mut params = self.base_params();
        params.retain(|p| p.starts_with("page="));
        let mut base = self.base_params();
        if p > 1 {
            base.push(format!("page={p}"));
        }
        Self::build_url(base)
    }
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

    Ok(Html(
        EventsTemplate {
            current_user,
            events,
            eras,
            filter_status: status.unwrap_or("").to_string(),
            filter_mode: mode_filter.unwrap_or("").to_string(),
            filter_era: era_filter,
            page,
            total_pages,
        }
        .render()
        .map_err(internal_error)?,
    ))
}
