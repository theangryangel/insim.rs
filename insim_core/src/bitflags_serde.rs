//! Serde (de)serialisation for bitflags types as arrays of active flag-name strings.
//!
//! Use with `#[cfg_attr(feature = "serde", serde(with = "insim_core::bitflags_serde"))]`
//! on any struct field or enum variant payload whose type is a `bitflags::Flags` type.

use serde::{Deserializer, Serializer};

/// Serialise a bitflags value as a JSON array of the names of active flags.
pub fn serialize<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    T: bitflags::Flags + Copy,
    S: Serializer,
{
    use serde::ser::SerializeSeq;
    let active: Vec<&str> = T::FLAGS
        .iter()
        .filter(|f| value.contains(*f.value()))
        .map(|f| f.name())
        .collect();
    let mut seq = serializer.serialize_seq(Some(active.len()))?;
    for name in &active {
        seq.serialize_element(name)?;
    }
    seq.end()
}

/// Deserialise a bitflags value from a JSON array of flag-name strings.
pub fn deserialize<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: bitflags::Flags + Copy + std::ops::BitOrAssign,
    D: Deserializer<'de>,
{
    use serde::de::Error as _;
    let names = <Vec<String> as serde::Deserialize>::deserialize(deserializer)?;
    let mut result = T::empty();
    for name in &names {
        let flag = T::FLAGS
            .iter()
            .find(|f| f.name() == name.as_str())
            .ok_or_else(|| D::Error::custom(format!("unknown flag: {name}")))?;
        result |= *flag.value();
    }
    Ok(result)
}
