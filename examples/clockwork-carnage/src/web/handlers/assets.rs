use axum::{extract::Path, http::StatusCode, response::IntoResponse, http::header::CONTENT_TYPE};

#[derive(rust_embed::Embed)]
#[folder = "assets/"]
struct Assets;

/// Serve static files from `/assets/*` path
pub async fn assets(Path(path): Path<String>) -> impl IntoResponse {
    match Assets::get(path.as_str()) {
        Some(content) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            ([(CONTENT_TYPE, mime.as_ref())], content.data).into_response()
        }
        None => (StatusCode::NOT_FOUND, "404 Not Found").into_response(),
    }
}
