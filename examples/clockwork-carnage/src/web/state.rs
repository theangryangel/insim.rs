use std::sync::Arc;

use axum::{
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
};
use oauth2::{AuthUrl, ClientId, CsrfToken, RedirectUrl, basic::BasicClient};
use tower_sessions::Session as TowerSession;

use crate::web::AuthSession;

#[derive(Clone)]
pub struct AppState {
    pub pool: Arc<crate::db::Pool>,
    pub oauth_client: BasicClient,
    pub presence: Option<kitcar::presence::Presence>,
}

pub struct PageCtx {
    pub current_user: Option<String>,
    pub admin: bool,
    pub csrf_token: String,
}

impl FromRequestParts<AppState> for PageCtx {
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, StatusCode> {
        let session = TowerSession::from_request_parts(parts, state)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let auth_session = AuthSession::from_request_parts(parts, state)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let current_user = auth_session.user.as_ref().map(|u| u.uname.clone());
        let admin = auth_session.user.map(|u| u.admin).unwrap_or(false);
        let csrf_token = get_or_create_csrf_token(&session).await?;
        Ok(PageCtx {
            current_user,
            admin,
            csrf_token,
        })
    }
}

pub async fn get_or_create_csrf_token(session: &TowerSession) -> Result<String, StatusCode> {
    if let Some(token) = session
        .get::<String>("csrf_token")
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    {
        return Ok(token);
    }
    let token = CsrfToken::new_random().secret().clone();
    session
        .insert("csrf_token", &token)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(token)
}

pub fn build_oauth_client(client_id: &str, redirect_uri: &str) -> anyhow::Result<BasicClient> {
    Ok(BasicClient::new(
        ClientId::new(client_id.to_string()),
        None,
        AuthUrl::new("https://id.lfs.net/oauth2/authorize".to_string())?,
        None,
    )
    .set_redirect_uri(RedirectUrl::new(redirect_uri.to_string())?))
}
