pub mod assets;
pub mod auth;
pub mod events;
pub mod index;
pub mod leaderboard;
pub mod profile;

pub use assets::*;
pub use auth::*;
pub use events::*;
pub use index::*;
pub use leaderboard::*;
pub use profile::*;

pub(crate) fn internal_error(e: impl std::fmt::Display) -> axum::http::StatusCode {
    tracing::error!(%e, "internal error");
    axum::http::StatusCode::INTERNAL_SERVER_ERROR
}
