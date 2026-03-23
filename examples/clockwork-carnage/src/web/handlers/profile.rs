use askama::Template;
use axum::{
    Form,
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Redirect},
};

use super::internal_error;
use crate::{
    db,
    web::{
        AuthSession,
        state::{AppState, PageCtx},
    },
};

#[derive(Template)]
#[template(path = "profile.html")]
pub struct ProfileTemplate {
    pub page: PageCtx,
    pub twitch_username: Option<String>,
    pub youtube_username: Option<String>,
}

pub async fn profile_get(
    page: PageCtx,
    auth_session: AuthSession,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, StatusCode> {
    if auth_session.user.is_none() {
        return Ok(Redirect::to("/login").into_response());
    }
    let user = db::get_user_by_id(&state.pool, auth_session.user.as_ref().unwrap().id)
        .await
        .map_err(internal_error)?;
    let (twitch_username, youtube_username) = user
        .map(|u| (u.twitch_username, u.youtube_username))
        .unwrap_or((None, None));
    Ok(Html(
        ProfileTemplate {
            page,
            twitch_username,
            youtube_username,
        }
        .render()
        .map_err(internal_error)?,
    )
    .into_response())
}

#[derive(serde::Deserialize)]
pub struct ProfileForm {
    pub csrf_token: String,
    pub twitch_username: String,
    pub youtube_username: String,
}

pub async fn profile_post(
    page: PageCtx,
    auth_session: AuthSession,
    State(state): State<AppState>,
    Form(form): Form<ProfileForm>,
) -> Result<impl IntoResponse, StatusCode> {
    let uname = match &auth_session.user {
        Some(u) => u.uname.clone(),
        None => return Ok(Redirect::to("/login").into_response()),
    };
    if form.csrf_token != page.csrf_token {
        return Err(StatusCode::FORBIDDEN);
    }
    let twitch = form.twitch_username.trim().trim_start_matches('@');
    let twitch_username = if twitch.is_empty() {
        None
    } else {
        Some(twitch)
    };
    let youtube = form.youtube_username.trim().trim_start_matches('@');
    let youtube_username = if youtube.is_empty() {
        None
    } else {
        Some(youtube)
    };
    db::update_user_profile(&state.pool, &uname, twitch_username, youtube_username)
        .await
        .map_err(internal_error)?;
    Ok(Redirect::to("/profile").into_response())
}
