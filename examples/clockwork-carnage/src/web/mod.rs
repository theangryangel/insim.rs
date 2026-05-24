//! Web module: axum-login auth backend + HTTP server.

pub mod changeset;
pub mod csrf;
pub mod filters;
pub mod handlers;
pub mod state;

use std::{net::SocketAddr, sync::Arc};

use axum::{Router, middleware, routing::get};
use axum_login::{AuthManagerLayerBuilder, AuthUser, AuthnBackend, UserId};
pub use changeset::{Changeset, empty_string_as_none};
use clap::Parser;
pub use csrf::Csrf;
use handlers::*;
use state::{AppState, build_oauth_client};
use tower_sessions::{
    SessionManagerLayer,
    cookie::{Key, SameSite},
};
use tower_sessions_sqlx_store::PostgresStore;

use crate::db;

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

        let user = db::upsert_user_with_token(&self.pool, uname, pname, access_token).await?;
        Ok(Some(user))
    }

    async fn get_user(&self, user_id: &UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        Ok(db::get_user_by_id(&self.pool, *user_id).await?)
    }
}

pub type AuthSession = axum_login::AuthSession<Backend>;

#[derive(Debug, Clone)]
pub struct WebConfig {
    pub base_url: String,
    pub oauth_client_id: String,
    pub oauth_client_secret: String,
    pub session_key: Option<String>,
}

#[derive(Parser, Debug)]
pub struct WebArgs {
    /// Postgres connection string.
    #[arg(long)]
    pub db: String,

    /// Listen address (host:port).
    #[arg(long, default_value = "0.0.0.0:3000")]
    pub listen: String,

    /// Base URL for OAuth redirect (e.g. https://example.com).
    #[arg(long, default_value = "http://localhost:3000")]
    pub base_url: String,

    /// LFS OAuth client ID.
    #[arg(long, env = "OAUTH_CLIENT_ID")]
    pub oauth_client_id: String,

    /// LFS OAuth client secret.
    #[arg(long, env = "OAUTH_CLIENT_SECRET")]
    pub oauth_client_secret: String,

    /// Session encryption key (hex or raw). If not set, a random key is used.
    #[arg(long, env = "SESSION_KEY")]
    pub session_key: Option<String>,
}

pub async fn run_web(args: WebArgs) -> anyhow::Result<()> {
    let pool = db::connect(&args.db).await?;
    let listen: SocketAddr = args.listen.parse()?;
    let cfg = WebConfig {
        base_url: args.base_url,
        oauth_client_id: args.oauth_client_id,
        oauth_client_secret: args.oauth_client_secret,
        session_key: args.session_key,
    };
    serve(listen, pool, cfg).await
}

pub async fn serve(listen: SocketAddr, pool: db::Pool, cfg: WebConfig) -> anyhow::Result<()> {
    let redirect_uri = format!("{}/auth/callback", cfg.base_url);
    let oauth_client = build_oauth_client(&cfg.oauth_client_id, &redirect_uri)?;
    let backend = Backend::new(
        pool.clone(),
        cfg.oauth_client_id,
        cfg.oauth_client_secret,
        redirect_uri,
    );

    let session_store = PostgresStore::new(pool.clone());
    session_store.migrate().await?;

    let key = match cfg.session_key {
        Some(ref k) => Key::from(k.as_bytes()),
        None => Key::generate(),
    };

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
        .route("/assets/{*file}", get(assets))
        .route("/profile", get(profile_get).post(profile_post))
        .route("/about", get(about))
        .route("/", get(index))
        .route("/events", get(events))
        .route("/events/new", get(event_new_get).post(event_new_post))
        .route("/events/{id}", get(event_detail))
        .route(
            "/events/{id}/edit",
            get(event_edit_get).post(event_edit_post),
        )
        .route("/login", get(login))
        .route("/auth/callback", get(callback))
        .route("/logout", get(logout))
        .route_layer(middleware::from_fn(csrf::csrf_protect))
        .layer(auth_layer)
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind(listen).await?;
    tracing::info!("Web listening on {listen}");

    axum::serve(listener, app).await?;

    Ok(())
}
