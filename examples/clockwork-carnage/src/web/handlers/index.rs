use axum::{extract::State, http::StatusCode, response::Html};

use super::internal_error;
use crate::{
    db,
    web::{
        AuthSession,
        state::{AppState, User},
        views,
    },
};

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
    Ok(Html(views::index(&current_user, &events).into_string()))
}
