//! Web module: axum-login auth backend + HTTP server.

pub mod handlers;
pub mod state;
pub mod filters;

use std::net::SocketAddr;
use std::sync::Arc;

use axum::Router;
use axum::routing::{get, post};
use axum_login::AuthManagerLayerBuilder;
use tower_sessions::{SessionManagerLayer, cookie::Key, cookie::SameSite};
use tower_sessions_sqlx_store::SqliteStore;

use crate::db;
use state::{AppState, build_oauth_client};
use handlers::*;

// -- axum-login auth backend --------------------------------------------------

use axum_login::{AuthUser, AuthnBackend, UserId};

impl AuthUser for db::User {
    type Id = i64;

    fn id(&self) -> Self::Id {
        self.id
    }

    fn session_auth_hash(&self) -> &[u8] {
        self.oauth_access_token
            .as_deref()
            .map(str::as_bytes)
            .unwrap_or(&[])
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct OAuthCredentials {
    pub code: String,
    #[allow(unused)]
    pub state: String,
}

#[derive(Clone)]
pub struct Backend {
    pub pool: db::Pool,
    http_client: reqwest::Client,
    client_id: String,
    client_secret: String,
    redirect_uri: String,
}

impl Backend {
    pub fn new(
        pool: db::Pool,
        client_id: String,
        client_secret: String,
        redirect_uri: String,
    ) -> Self {
        Self {
            pool,
            http_client: reqwest::Client::new(),
            client_id,
            client_secret,
            redirect_uri,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum BackendError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error("OAuth error: {0}")]
    OAuth(String),
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error("missing username in LFS token response")]
    MissingUname,
}

impl AuthnBackend for Backend {
    type User = db::User;
    type Credentials = OAuthCredentials;
    type Error = BackendError;

    async fn authenticate(
        &self,
        creds: Self::Credentials,
    ) -> Result<Option<Self::User>, Self::Error> {
        // Exchange code for token via raw POST — LFS returns text/html Content-Type
        // even for JSON responses, so we bypass the oauth2 crate's content-type check.
        let resp = self
            .http_client
            .post("https://id.lfs.net/oauth2/access_token")
            .form(&[
                ("grant_type", "authorization_code"),
                ("code", &creds.code),
                ("redirect_uri", &self.redirect_uri),
                ("client_id", &self.client_id),
                ("client_secret", &self.client_secret),
            ])
            .send()
            .await?;

        let body = resp.bytes().await?;
        let token_data: serde_json::Value = serde_json::from_slice(&body)
            .map_err(|e| BackendError::OAuth(format!("failed to parse token response: {e}")))?;

        let access_token = token_data["access_token"]
            .as_str()
            .ok_or_else(|| BackendError::OAuth(format!("missing access_token: {token_data}")))?;

        // Fetch user info from the LFS API
        let userinfo_resp = self
            .http_client
            .get("https://api.lfs.net/userinfo")
            .bearer_auth(access_token)
            .send()
            .await?;

        let userinfo_body = userinfo_resp.bytes().await?;
        let userinfo: serde_json::Value = serde_json::from_slice(&userinfo_body)
            .map_err(|e| BackendError::OAuth(format!("failed to parse userinfo response: {e}")))?;

        let data = &userinfo["data"];

        let uname = data["preferred_username"]
            .as_str()
            .or_else(|| data["username"].as_str())
            .ok_or(BackendError::MissingUname)?;

        let pname = data["name"].as_str().unwrap_or(uname);

        let user =
            db::upsert_user_with_token(&self.pool, uname, pname, access_token).await?;
        Ok(Some(user))
    }

    async fn get_user(&self, user_id: &UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        Ok(db::get_user_by_id(&self.pool, *user_id).await?)
    }
}

// Convenience
pub type AuthSession = axum_login::AuthSession<Backend>;

// -- Web configuration --------------------------------------------------------

#[derive(Debug, Clone)]
pub struct WebConfig {
    pub oauth_client_id: String,
    pub oauth_client_secret: String,
    pub oauth_redirect_uri: String,
    pub session_key: String,
}

// -- Server entry point -------------------------------------------------------

pub async fn serve(listen: SocketAddr, pool: db::Pool, cfg: WebConfig) -> anyhow::Result<()> {
    let oauth_client = build_oauth_client(&cfg.oauth_client_id, &cfg.oauth_redirect_uri)?;
    let backend = Backend::new(
        pool.clone(),
        cfg.oauth_client_id,
        cfg.oauth_client_secret,
        cfg.oauth_redirect_uri,
    );

    let session_store = SqliteStore::new(pool.clone());
    session_store.migrate().await?;

    let key = Key::from(cfg.session_key.as_bytes());

    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_same_site(SameSite::Lax)
        .with_private(key);

    let auth_layer = AuthManagerLayerBuilder::new(backend, session_layer).build();

    let app_state = AppState {
        pool: Arc::new(pool),
        oauth_client,
    };

    let app = Router::new()
        .route("/logo.svg", get(logo))
        .route("/", get(index))
        .route("/events", get(events))
        .route("/events/new", get(event_new_get).post(event_new_post))
        .route("/events/{id}", get(event_detail))
        .route("/events/{id}/edit", get(event_edit_get).post(event_edit_post))
        .route("/events/{id}/standings", get(event_standings))
        .route("/events/{id}/rounds/{round}", get(event_round))
        .route("/events/{id}/standings/best", get(event_standings_best))
        .route("/events/{id}/standings/all", get(event_standings_all))
        .route("/events/{id}/standings/bomb/best", get(event_bomb_best))
        .route("/events/{id}/standings/bomb/all", get(event_bomb_all))
        .route("/events/{id}/standings/climb/best", get(event_climb_best))
        .route("/events/{id}/standings/climb/all", get(event_climb_all))
        .route("/events/{id}/start", post(event_start))
        .route("/events/{id}/complete", post(event_complete))
        .route("/events/{id}/cancel", post(event_cancel))
        .route("/login", get(login))
        .route("/auth/callback", get(callback))
        .route("/logout", get(logout))
        .layer(auth_layer)
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind(listen).await?;
    tracing::info!("Web listening on {listen}");

    axum::serve(listener, app).await?;

    Ok(())
}
