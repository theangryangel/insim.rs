use axum::http::StatusCode;

pub async fn logo() -> (StatusCode, [(&'static str, &'static str); 1], &'static [u8]) {
    (
        StatusCode::OK,
        [("content-type", "image/svg+xml")],
        include_bytes!("../../../logo.svg"),
    )
}
