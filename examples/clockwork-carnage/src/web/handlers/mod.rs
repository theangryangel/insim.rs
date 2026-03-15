pub mod assets;
pub mod auth;
pub mod index;
pub mod events;

pub use assets::*;
pub use auth::*;
pub use index::*;
pub use events::*;

pub(crate) fn internal_error(e: impl std::fmt::Display) -> axum::http::StatusCode {
    tracing::error!("{e:#}");
    axum::http::StatusCode::INTERNAL_SERVER_ERROR
}
