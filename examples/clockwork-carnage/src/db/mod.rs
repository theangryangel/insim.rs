//! Postgres database layer for Clockwork Carnage.

use insim::core::track::Track;
use jiff_sqlx::Timestamp;
use sqlx::types::Json;

pub type Pool = sqlx::PgPool;

pub async fn connect(url: &str) -> Result<Pool, sqlx::Error> {
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(url)
        .await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    Ok(pool)
}

fn default_checkpoint_penalty_ms() -> i64 {
    250
}

fn default_collision_max_penalty_ms() -> i64 {
    500
}

#[derive(
    Debug, Clone, PartialEq, Eq, Default, sqlx::Type, serde::Serialize, serde::Deserialize,
)]
#[sqlx(type_name = "text", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum EventStatus {
    #[default]
    Pending,
    Live,
    Completed,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EventMode {
    Metronome {
        target_ms: i64,
    },
    Shortcut,
    Bomb {
        checkpoint_timeout_secs: i64,
        #[serde(default = "default_checkpoint_penalty_ms")]
        checkpoint_penalty_ms: i64,
        #[serde(default = "default_collision_max_penalty_ms")]
        collision_max_penalty_ms: i64,
    },
}

impl EventMode {
    pub fn label(&self) -> &'static str {
        match self {
            EventMode::Shortcut => "Shortcut",
            EventMode::Metronome { .. } => "Metronome",
            EventMode::Bomb { .. } => "Bomb Run",
        }
    }

    pub fn colour(&self) -> &'static str {
        match self {
            EventMode::Shortcut => "#22c55e",
            EventMode::Metronome { .. } => "#3b82f6",
            EventMode::Bomb { .. } => "#f59e0b",
        }
    }

    pub fn tag_classes(&self) -> String {
        format!("bg-[{}]/15 text-[{}]", self.colour(), self.colour())
    }
}

#[derive(Clone, sqlx::FromRow)]
pub struct User {
    pub id: i64,
    pub uname: String,
    pub pname: String,
    pub last_seen: Timestamp,
    pub oauth_access_token: Option<String>,
    pub admin: bool,
    pub twitch_username: Option<String>,
    pub youtube_username: Option<String>,
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

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Era {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Event {
    pub id: i64,
    pub status: EventStatus,
    pub mode: Json<EventMode>,
    #[sqlx(try_from = "String")]
    pub track: Track,
    pub layout: String,
    pub ended_at: Option<Timestamp>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub writeup: Option<String>,
    pub scheduled_at: Option<Timestamp>,
    pub allowed_vehicles: Json<Vec<String>>,
    pub era_id: Option<i64>,
    pub era_name: Option<String>,
}

#[derive(Debug, Clone)]
pub struct MetronomeStanding {
    pub uname: String,
    pub pname: String,
    pub best_delta_ms: i64,
    pub twitch_username: Option<String>,
    pub youtube_username: Option<String>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ShortcutTime {
    pub uname: String,
    pub pname: String,
    pub vehicle: String,
    pub time_ms: i64,
    pub set_at: Timestamp,
    pub twitch_username: Option<String>,
    pub youtube_username: Option<String>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct BombRun {
    pub uname: String,
    pub pname: String,
    pub vehicle: String,
    pub checkpoint_count: i64,
    pub survival_ms: i64,
    pub recorded_at: Timestamp,
    pub twitch_username: Option<String>,
    pub youtube_username: Option<String>,
}

mod bomb;
mod events;
mod metronome;
mod shortcut;
mod users;

pub use bomb::*;
pub use events::*;
pub use metronome::*;
pub use shortcut::*;
pub use users::*;
