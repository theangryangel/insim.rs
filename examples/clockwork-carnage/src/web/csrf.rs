use axum::{
    body::Body,
    extract::{FromRequestParts, Request},
    http::{Method, StatusCode, request::Parts},
    middleware::Next,
    response::Response,
};
use oauth2::CsrfToken;
use tower_sessions::Session;

use crate::web::state::AppState;

pub const CSRF_SESSION_KEY: &str = "csrf_token";

/// Carries the CSRF token for the current session. Use this extractor in any
/// handler that renders a form - it creates the token on first use and exposes
/// it for the template's hidden field.
pub struct Csrf {
    pub token: String,
}

impl FromRequestParts<AppState> for Csrf {
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, StatusCode> {
        let session = Session::from_request_parts(parts, state)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let token = if let Some(t) = session
            .get::<String>(CSRF_SESSION_KEY)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        {
            t
        } else {
            let t = CsrfToken::new_random().secret().clone();
            session
                .insert(CSRF_SESSION_KEY, &t)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            t
        };

        Ok(Csrf { token })
    }
}

pub async fn csrf_protect(
    session: Session,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    if matches!(
        req.method(),
        &Method::GET | &Method::HEAD | &Method::OPTIONS | &Method::TRACE
    ) {
        return Ok(next.run(req).await);
    }

    let session_token: Option<String> = session
        .get(CSRF_SESSION_KEY)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let Some(expected) = session_token.filter(|t| !t.is_empty()) else {
        return Err(StatusCode::FORBIDDEN);
    };

    // Header takes priority - used by fetch/XHR requests.
    if let Some(header_val) = req
        .headers()
        .get("x-csrf-token")
        .and_then(|v| v.to_str().ok())
    {
        return if header_val == expected {
            Ok(next.run(req).await)
        } else {
            Err(StatusCode::FORBIDDEN)
        };
    }

    // Fall back to form body (standard HTML form submission).
    let (parts, body) = req.into_parts();
    let bytes = axum::body::to_bytes(body, 16 * 1024 * 1024)
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let form_token = extract_csrf_from_body(&bytes);
    if form_token.as_deref() != Some(expected.as_str()) {
        return Err(StatusCode::FORBIDDEN);
    }

    let req = Request::from_parts(parts, Body::from(bytes));
    Ok(next.run(req).await)
}

fn extract_csrf_from_body(bytes: &[u8]) -> Option<String> {
    #[derive(serde::Deserialize, Default)]
    struct CsrfField {
        #[serde(default)]
        csrf_token: String,
    }
    serde_urlencoded::from_bytes::<CsrfField>(bytes)
        .ok()
        .filter(|f| !f.csrf_token.is_empty())
        .map(|f| f.csrf_token)
}
