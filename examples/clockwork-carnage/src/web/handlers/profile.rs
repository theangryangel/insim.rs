use askama::Template;
use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Redirect, Response},
};
use validator::Validate;

use super::internal_error;
use crate::{
    db,
    web::{
        AuthSession, Changeset, Csrf,
        state::{AppState, User},
    },
};

#[derive(Template)]
#[template(path = "profile.html")]
pub struct ProfileTemplate {
    pub current_user: User,
    pub csrf_token: String,
    pub cs: Changeset<ProfileInput>,
}

#[derive(serde::Deserialize, Default, Validate)]
pub struct ProfileInput {
    pub twitch_username: String,
    pub youtube_username: String,
}

pub async fn profile_get(
    auth: AuthSession,
    csrf: Csrf,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, StatusCode> {
    let current_user = User::from(&auth);
    if auth.user.is_none() {
        return Ok(Redirect::to("/login").into_response());
    }
    let user = db::get_user_by_id(&state.pool, auth.user.as_ref().unwrap().id)
        .await
        .map_err(internal_error)?;
    let (twitch_username, youtube_username) = user
        .map(|u| {
            (
                u.twitch_username.unwrap_or_default(),
                u.youtube_username.unwrap_or_default(),
            )
        })
        .unwrap_or_default();
    let cs = Changeset::new(ProfileInput {
        twitch_username,
        youtube_username,
    });
    Ok(Html(
        ProfileTemplate {
            current_user,
            csrf_token: csrf.token,
            cs,
        }
        .render()
        .map_err(internal_error)?,
    )
    .into_response())
}

pub async fn profile_post(
    auth: AuthSession,
    State(state): State<AppState>,
    cs: Changeset<ProfileInput>,
) -> Result<Response, StatusCode> {
    let uname = match &auth.user {
        Some(u) => u.uname.clone(),
        None => return Ok(Redirect::to("/login").into_response()),
    };
    let twitch = cs.params.twitch_username.trim().trim_start_matches('@');
    let twitch_username = if twitch.is_empty() {
        None
    } else {
        Some(twitch)
    };
    let youtube = cs.params.youtube_username.trim().trim_start_matches('@');
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
