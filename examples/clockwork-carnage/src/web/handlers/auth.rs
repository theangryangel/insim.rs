use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
};
use oauth2::{CsrfToken, Scope};
use tower_sessions::Session as TowerSession;

use crate::web::{AuthSession, OAuthCredentials, state::AppState};

#[derive(serde::Deserialize)]
pub struct AuthzResp {
    pub code: String,
    pub state: String,
}

pub async fn login(
    auth_session: AuthSession,
    session: TowerSession,
    State(state): State<AppState>,
) -> Result<Response, StatusCode> {
    if auth_session.user.is_some() {
        return Ok(Redirect::to("/").into_response());
    }
    let (auth_url, csrf_token) = state
        .oauth_client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("openid".to_string()))
        .add_scope(Scope::new("profile".to_string()))
        .url();
    session
        .insert("oauth_csrf_state", csrf_token.secret().clone())
        .await
        .map_err(|e| {
            tracing::error!("failed to insert csrf state into session: {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    Ok(Redirect::to(auth_url.as_str()).into_response())
}

pub async fn callback(
    mut auth_session: AuthSession,
    session: TowerSession,
    Query(params): Query<AuthzResp>,
) -> Result<Response, StatusCode> {
    let expected: Option<String> = session.remove("oauth_csrf_state").await.map_err(|e| {
        tracing::error!("failed to remove csrf state from session: {e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    if expected.as_deref() != Some(&params.state) {
        return Ok(StatusCode::BAD_REQUEST.into_response());
    }
    let creds = OAuthCredentials {
        code: params.code,
        state: params.state,
    };
    match auth_session.authenticate(creds).await {
        Ok(Some(user)) => {
            auth_session.login(&user).await.map_err(|e| {
                tracing::error!("failed to log in session after authentication: {e}");
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
            Ok(Redirect::to("/").into_response())
        },
        Ok(None) => Ok(StatusCode::UNAUTHORIZED.into_response()),
        Err(e) => {
            tracing::error!("Auth error: {e}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        },
    }
}

pub async fn logout(mut auth_session: AuthSession) -> Result<Redirect, StatusCode> {
    let _ = auth_session.logout().await.map_err(|e| {
        tracing::error!("failed to destroy session on logout: {e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    Ok(Redirect::to("/"))
}
