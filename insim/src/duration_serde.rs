//! Serde/schemars helpers for representing `std::time::Duration` as
//! milliseconds.
//!
//! Use with `#[cfg_attr(feature = "serde", serde(with = "insim::duration_serde"))]`
//! on a `Duration` field. For `Option<Duration>`, use the `option` submodule:
//! `serde(with = "insim::duration_serde::option")`.
//!
//! The InSim binary protocol always uses millisecond-scale durations, so
//! this keeps the JSON representation aligned with the underlying values.

use std::time::Duration;

use serde::{Deserialize, Deserializer, Serializer};

/// Serialise a `Duration` as integer milliseconds.
pub fn serialize<S: Serializer>(d: &Duration, s: S) -> Result<S::Ok, S::Error> {
    // Saturate to u64 - Durations carrying more ms than fits in u64 are
    // not representable on the InSim wire anyway (max protocol field is
    // u32 ms).
    let ms: u64 = d.as_millis().try_into().unwrap_or(u64::MAX);
    s.serialize_u64(ms)
}

/// Deserialise a `Duration` from integer milliseconds.
pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Duration, D::Error> {
    Ok(Duration::from_millis(u64::deserialize(d)?))
}

/// Serde adaptor for `Option<Duration>` <-> integer milliseconds or null.
pub mod option {
    use super::*;

    /// Serialise an `Option<Duration>` as integer milliseconds or null.
    pub fn serialize<S: Serializer>(d: &Option<Duration>, s: S) -> Result<S::Ok, S::Error> {
        match d {
            Some(d) => super::serialize(d, s),
            None => s.serialize_none(),
        }
    }

    /// Deserialise an `Option<Duration>` from integer milliseconds or null.
    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Option<Duration>, D::Error> {
        Ok(Option::<u64>::deserialize(d)?.map(Duration::from_millis))
    }
}
