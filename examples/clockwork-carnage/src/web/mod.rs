//! axum-login auth backend for LFS OAuth2.

use axum_login::{AuthUser, AuthnBackend, UserId};

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
