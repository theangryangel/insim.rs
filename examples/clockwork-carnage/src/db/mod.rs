//! SQLite database layer for Clockwork Carnage.

use std::{fmt, str::FromStr};

use insim::core::{track::Track, vehicle::Vehicle};
use sqlx::{
    FromRow,
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions}, types::Json,
};

pub type Pool = sqlx::SqlitePool;

pub async fn connect(path: &str) -> Result<Pool, sqlx::Error> {
    let options = SqliteConnectOptions::from_str(path)?
        .journal_mode(SqliteJournalMode::Wal)
        .create_if_missing(true);
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    Ok(pool)
}

// -- Enums --------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EventMode {
    Metronome {
        target_ms: i64,
    },
    Shortcut,
    Bomb {
        checkpoint_timeout_secs: i64,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventStatus {
    Pending,
    Active,
    Completed,
    Cancelled,
}

impl fmt::Display for EventStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Pending => f.write_str("PENDING"),
            Self::Active => f.write_str("ACTIVE"),
            Self::Completed => f.write_str("COMPLETED"),
            Self::Cancelled => f.write_str("CANCELLED"),
        }
    }
}

impl TryFrom<String> for EventStatus {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.as_str() {
            "PENDING" => Ok(Self::Pending),
            "ACTIVE" => Ok(Self::Active),
            "COMPLETED" => Ok(Self::Completed),
            "CANCELLED" => Ok(Self::Cancelled),
            other => Err(format!("unknown event status: {other}")),
        }
    }
}

// -- Row types ----------------------------------------------------------------

#[derive(Clone, FromRow)]
pub struct User {
    pub id: i64,
    pub uname: String,
    pub pname: String,
    pub last_seen: String,
    pub oauth_access_token: Option<String>,
    pub admin: bool,
}

impl std::fmt::Debug for User {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("User")
            .field("id", &self.id)
            .field("uname", &self.uname)
            .field("pname", &self.pname)
            .field("last_seen", &self.last_seen)
            .field("oauth_access_token", &"[redacted]")
            .finish()
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct Event {
    pub id: i64,
    pub mode: Json<EventMode>,
    #[sqlx(try_from = "String")]
    pub status: EventStatus,
    #[sqlx(try_from = "String")]
    pub track: Track,
    pub layout: String,
    #[allow(unused)]
    pub created_at: String,
    #[allow(unused)]
    pub started_at: Option<String>,
    #[allow(unused)]
    pub ended_at: Option<String>,
    pub scheduled_at: Option<String>,
    #[allow(unused)]
    pub scheduled_end_at: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub writeup: Option<String>,
    pub allowed_vehicles: Json<Vec<Vehicle>>,
}


#[derive(Debug, Clone)]
pub struct MetronomeStanding {
    pub uname: String,
    pub pname: String,
    pub best_delta_ms: i64,
}

#[derive(Debug, Clone, FromRow)]
pub struct ShortcutTime {
    #[allow(unused)]
    pub id: i64,
    #[allow(unused)]
    pub event_id: i64,
    pub uname: String,
    pub pname: String,
    pub vehicle: String,
    pub time_ms: i64,
    pub set_at: String,
}

#[derive(Debug, Clone, FromRow)]
pub struct BombRun {
    #[allow(unused)]
    pub id: i64,
    #[allow(unused)]
    pub event_id: i64,
    #[allow(unused)]
    pub user_id: i64,
    pub uname: String,
    pub pname: String,
    pub vehicle: String,
    pub checkpoint_count: i64,
    pub survival_ms: i64,
    pub recorded_at: String,
}

// -- Submodules ---------------------------------------------------------------

mod events;
mod users;
mod metronome;
mod shortcut;
mod bomb;

pub use events::*;
pub use users::*;
pub use metronome::*;
pub use shortcut::*;
pub use bomb::*;
