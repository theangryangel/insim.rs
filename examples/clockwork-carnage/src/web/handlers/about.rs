use askama::Template;
use axum::{http::StatusCode, response::Html};

use crate::web::{AuthSession, state::User};

#[derive(Template)]
#[template(path = "about.html")]
pub struct AboutTemplate {
    pub current_user: User,
}

pub async fn about(auth: AuthSession) -> Result<Html<String>, StatusCode> {
    let tmpl = AboutTemplate {
        current_user: User::from(&auth),
    };
    Ok(Html(tmpl.render().map_err(super::internal_error)?))
}
