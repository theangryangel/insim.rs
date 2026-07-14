mod detail;
mod edit;
mod list;
mod new;

pub use detail::*;
pub use edit::*;
pub use list::*;
pub use new::*;

pub(super) use super::internal_error;

fn parse_datetime_local(s: &str) -> Result<jiff::Timestamp, axum::http::StatusCode> {
    let rfc3339 = match s.len() {
        16 => format!("{s}:00Z"),
        19 => format!("{s}Z"),
        _ => return Err(axum::http::StatusCode::BAD_REQUEST),
    };
    rfc3339
        .parse::<jiff::Timestamp>()
        .map_err(|_| axum::http::StatusCode::BAD_REQUEST)
}
