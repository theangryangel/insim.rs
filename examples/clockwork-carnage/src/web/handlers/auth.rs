use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect},
};
use oauth2::{CsrfToken, Scope};
use tower_sessions::Session as TowerSession;

use crate::web::{AuthSession, OAuthCredentials};
use crate::web::state::AppState;

#[derive(serde::Deserialize)]
pub struct AuthzResp {
    pub code: String,
    pub state: String,
}

pub async fn login(
    auth_session: AuthSession,
    session: TowerSession,
    State(state): State<AppState>,
) -> impl IntoResponse {
    if auth_session.user.is_some() {
        return Redirect::to("/").into_response();
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
        .unwrap();
    Redirect::to(auth_url.as_str()).into_response()
}

pub async fn callback(
    mut auth_session: AuthSession,
    session: TowerSession,
    Query(params): Query<AuthzResp>,
) -> impl IntoResponse {
    let expected: Option<String> = session.remove("oauth_csrf_state").await.unwrap();
    if expected.as_deref() != Some(&params.state) {
        return StatusCode::BAD_REQUEST.into_response();
    }
    let creds = OAuthCredentials { code: params.code, state: params.state };
    match auth_session.authenticate(creds).await {
        Ok(Some(user)) => {
            auth_session.login(&user).await.unwrap();
            Redirect::to("/").into_response()
        }
        Ok(None) => StatusCode::UNAUTHORIZED.into_response(),
        Err(e) => {
            tracing::error!("Auth error: {e}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub async fn logout(mut auth_session: AuthSession) -> impl IntoResponse {
    let _ = auth_session.logout().await.unwrap();
    Redirect::to("/")
}
