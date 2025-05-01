#![doc = include_str!("../README.md")]

pub mod decode;
pub mod encode;
pub mod game_version;
pub mod license;
pub mod point;
pub mod speed;
pub mod string;
pub mod track;
pub mod vehicle;
pub mod wind;

use std::{fmt::Display, num::TryFromIntError};

pub use decode::Decode;
pub use encode::Encode;
use game_version::GameVersionParseError;
pub use string::{Ascii, Codepage};

// FIXME: rename, add line/contextual information, split into ReadBufError and WriteBufError
#[non_exhaustive]
/// Read/Write Error
#[derive(Debug)]
pub enum Error {
    /// Bad Magic
    BadMagic {
        /// found
        found: Box<dyn core::fmt::Debug + Send + Sync>,
    },
    /// No Variant
    NoVariantMatch {
        /// found
        found: u64,
    },
    /// Cannot convert
    NotAsciiChar {
        /// Found character
        found: char,
    },
    /// String is not completely Ascii
    NotAsciiString,
    /// TryFromInt
    TryFromInt(TryFromIntError),
    /// Value too large for field
    TooLarge,
    /// Game Version Parse Error
    GameVersionParseError(GameVersionParseError),
}

impl std::error::Error for Error {}

impl From<GameVersionParseError> for Error {
    fn from(value: GameVersionParseError) -> Self {
        Self::GameVersionParseError(value)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self) // FIXME
    }
}
