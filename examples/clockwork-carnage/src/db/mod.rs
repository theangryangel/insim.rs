//! SQLite database layer for Clockwork Carnage.

use std::{fmt, str::FromStr};

use insim::core::{track::Track, vehicle::Vehicle};
use sqlx::{
    FromRow,
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions},
    types::Json,
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

fn default_checkpoint_penalty_ms() -> i64 {
    250
}

fn default_collision_max_penalty_ms() -> i64 {
    500
}

// -- Timestamp newtype --------------------------------------------------------

/// A UTC timestamp stored as ISO 8601 text in SQLite.
/// Wraps `jiff::Timestamp` and provides sqlx encode/decode via RFC 3339 strings.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Timestamp(pub jiff::Timestamp);

impl Timestamp {
    pub fn now() -> Self {
        Timestamp(jiff::Timestamp::now())
    }
}

impl std::fmt::Debug for Timestamp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Timestamp({})", self.0)
    }
}

impl std::ops::Deref for Timestamp {
    type Target = jiff::Timestamp;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<jiff::Timestamp> for Timestamp {
    fn from(ts: jiff::Timestamp) -> Self {
        Timestamp(ts)
    }
}

impl From<Timestamp> for jiff::Timestamp {
    fn from(ts: Timestamp) -> Self {
        ts.0
    }
}

// sqlx type info: stored as TEXT in SQLite
impl sqlx::Type<sqlx::Sqlite> for Timestamp {
    fn type_info() -> sqlx::sqlite::SqliteTypeInfo {
        <String as sqlx::Type<sqlx::Sqlite>>::type_info()
    }
    fn compatible(ty: &sqlx::sqlite::SqliteTypeInfo) -> bool {
        <String as sqlx::Type<sqlx::Sqlite>>::compatible(ty)
    }
}

// Decode: read TEXT from SQLite, parse as RFC 3339
impl<'r> sqlx::Decode<'r, sqlx::Sqlite> for Timestamp {
    fn decode(
        value: sqlx::sqlite::SqliteValueRef<'r>,
    ) -> Result<Self, sqlx::error::BoxDynError> {
        let s = <&str as sqlx::Decode<sqlx::Sqlite>>::decode(value)?;
        let ts: jiff::Timestamp = s.parse().map_err(|_| {
            format!("invalid timestamp: {s:?}")
        })?;
        Ok(Timestamp(ts))
    }
}

// Encode: write RFC 3339 TEXT to SQLite
impl<'q> sqlx::Encode<'q, sqlx::Sqlite> for Timestamp {
    fn encode_by_ref(
        &self,
        args: &mut Vec<sqlx::sqlite::SqliteArgumentValue<'q>>,
    ) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>> {
        let s = self.0.to_string(); // RFC 3339 via jiff's Display impl
        args.push(sqlx::sqlite::SqliteArgumentValue::Text(
            std::borrow::Cow::Owned(s),
        ));
        Ok(sqlx::encode::IsNull::No)
    }
}

// -- Enums --------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EventMode {
    Metronome { target_ms: i64 },
    Shortcut,
    Bomb {
        checkpoint_timeout_secs: i64,
        #[serde(default = "default_checkpoint_penalty_ms")]
        checkpoint_penalty_ms: i64,
        #[serde(default = "default_collision_max_penalty_ms")]
        collision_max_penalty_ms: i64,
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

#[derive(Debug, Clone, FromRow)]
pub struct Event {
    pub id: i64,
    pub mode: Json<EventMode>,
    #[sqlx(try_from = "String")]
    pub status: EventStatus,
    #[sqlx(try_from = "String")]
    pub track: Track,
    pub layout: String,
    pub scheduled_at: Option<Timestamp>,
    pub scheduled_end_at: Option<Timestamp>,
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
    pub twitch_username: Option<String>,
    pub youtube_username: Option<String>,
}

#[derive(Debug, Clone, FromRow)]
pub struct ShortcutTime {
    pub uname: String,
    pub pname: String,
    pub vehicle: String,
    pub time_ms: i64,
    pub set_at: Timestamp,
    pub twitch_username: Option<String>,
    pub youtube_username: Option<String>,
}

#[derive(Debug, Clone, FromRow)]
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

// -- Submodules ---------------------------------------------------------------

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
