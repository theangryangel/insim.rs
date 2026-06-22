use axum::{http::StatusCode, response::Html};

use crate::web::{AuthSession, state::User, views};

pub async fn about(auth: AuthSession) -> Result<Html<String>, StatusCode> {
    Ok(Html(views::about(&User::from(&auth)).into_string()))
}
