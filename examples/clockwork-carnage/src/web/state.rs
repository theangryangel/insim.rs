use std::sync::Arc;

use oauth2::{AuthUrl, ClientId, RedirectUrl, basic::BasicClient};

use crate::web::AuthSession;

#[derive(Clone)]
pub struct AppState {
    pub pool: Arc<crate::db::Pool>,
    pub oauth_client: BasicClient,
}

pub struct User {
    pub uname: Option<String>,
    pub admin: bool,
}

impl From<&AuthSession> for User {
    fn from(auth: &AuthSession) -> Self {
        User {
            uname: auth.user.as_ref().map(|u| u.uname.clone()),
            admin: auth.user.as_ref().map(|u| u.admin).unwrap_or(false),
        }
    }
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
