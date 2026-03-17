pub mod assets;
pub mod auth;
pub mod events;
pub mod index;

pub use assets::*;
pub use auth::*;
pub use events::*;
pub use index::*;

pub(crate) fn internal_error(e: impl std::fmt::Display) -> axum::http::StatusCode {
    tracing::error!("{e:#}");
    axum::http::StatusCode::INTERNAL_SERVER_ERROR
}
