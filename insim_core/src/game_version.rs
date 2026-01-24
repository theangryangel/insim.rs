//! Tools for parsing, comparing and sorting a game version, based on best effort of known LFS
//! version

use std::{cmp::Ordering, fmt::Display, str::FromStr};

use bytes::Bytes;
use itertools::Itertools;

use crate::{Decode, Encode, EncodeString};

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
/// Possible errors when parsing a game version
pub enum GameVersionParseError {
    /// Could not parse a float
    #[error("Could not parse major version: {0}")]
    Major(String),

    /// Could not parse minor
    #[error("Could not parse minor version: {0}")]
    Minor(String),

    /// Could not parse an int
    #[error("Could not parse patch: {0}")]
    Patch(String),

    /// Could not parse string as UTF-8
    #[error("Could not parse string: {0:?}")]
    NotUtf8String(Bytes),
}

/// GameVersion
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct GameVersion {
    /// Version
    // XXX: Why a float? Because as far as I can tell Scawen treats LFS versions like a number, not
    // as a version, based on the existence of 0.04k. Version numbers tend not to have leading
    // zeros.
    pub major: f32,

    /// Patch
    pub minor: char,

    /// Patch revision
    pub patch: Option<usize>,
}

impl PartialEq for GameVersion {
    fn eq(&self, other: &Self) -> bool {
        self.major.to_bits() == other.major.to_bits()
            && self.minor == other.minor
            && self.patch.unwrap_or(0) == other.patch.unwrap_or(0)
    }
}

impl Eq for GameVersion {}

impl PartialOrd for GameVersion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for GameVersion {
    fn cmp(&self, other: &Self) -> Ordering {
        let major = self.major.partial_cmp(&other.major);
        let minor = self.minor.partial_cmp(&other.minor);
        let patch = self
            .patch
            .unwrap_or(0)
            .partial_cmp(&other.patch.unwrap_or(0));

        match (major, minor, patch) {
            (Some(Ordering::Equal), Some(Ordering::Equal), Some(patch_eq)) => patch_eq,
            (Some(Ordering::Equal), Some(Ordering::Greater), _) => Ordering::Greater,
            (Some(Ordering::Equal), Some(Ordering::Less), _) => Ordering::Less,

            (Some(non_eq), _, _) => non_eq,

            _ => Ordering::Equal,
        }
    }
}

impl Default for GameVersion {
    fn default() -> Self {
        Self {
            major: 0.0,
            minor: 'A',
            patch: None,
        }
    }
}

impl Display for GameVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(patch) = self.patch {
            write!(f, "{}{}{}", self.major, self.minor, patch)
        } else {
            write!(f, "{}{}", self.major, self.minor)
        }
    }
}

enum Position {
    Major,
    Minor,
    Patch,
}

impl FromStr for GameVersion {
    type Err = GameVersionParseError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let mut data = Self::default();
        let mut pos = Position::Major;
        let mut iter = text.chars().peekable();

        while iter.peek().is_some() {
            match pos {
                Position::Major => {
                    let major: String = iter
                        .take_while_ref(|x| x.is_numeric() || *x == '.')
                        .collect();
                    data.major = major
                        .parse()
                        .map_err(|e| GameVersionParseError::Major(format!("{}", e)))?;
                    pos = Position::Minor;
                },
                Position::Minor => {
                    let next = iter.next();

                    if let Some(patch) = next
                        && patch.is_ascii_alphabetic()
                    {
                        data.minor = patch.to_ascii_uppercase();
                        pos = Position::Patch;
                    } else {
                        return Err(GameVersionParseError::Minor(format!(
                            "Expected A-Z character, found {:?}",
                            next
                        )));
                    }
                },
                Position::Patch => {
                    let rev: String = iter.by_ref().take_while_ref(|x| x.is_numeric()).collect();
                    data.patch = Some(
                        rev.parse()
                            .map_err(|e| GameVersionParseError::Patch(format!("{}", e)))?,
                    );
                },
            }
        }

        Ok(data)
    }
}

impl Decode for GameVersion {
    fn decode(buf: &mut bytes::Bytes) -> Result<Self, crate::DecodeError> {
        let new = buf.split_to(8);

        match std::str::from_utf8(&new) {
            Ok(s) => GameVersion::from_str(s.trim_end_matches('\0'))
                .map_err(|e| crate::DecodeErrorKind::from(e).into()),
            Err(_) => Err(crate::DecodeErrorKind::GameVersionParseError(
                GameVersionParseError::NotUtf8String(new.clone()),
            )
            .into()),
        }
    }
}

impl Encode for GameVersion {
    fn encode(&self, buf: &mut bytes::BytesMut) -> Result<(), crate::EncodeError> {
        let ver = self.to_string();
        ver.encode_ascii(buf, 8, false)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use bytes::BytesMut;

    use super::*;

    #[test]
    fn test_all_known_versions() {
        let version = vec![
            "0.7F", "0.7E15", "0.7E14", "0.7E13", "0.7E12", "0.7E11", "0.7E10", "0.7E9", "0.7E8",
            "0.7E7", "0.7E6", "0.7E5", "0.7E4", "0.7E3", "0.7E2", "0.7E", "0.7D64", "0.7D63",
            "0.7D62", "0.7D61", "0.7D60", "0.7D59", "0.7D58", "0.7D57", "0.7D56", "0.7D55",
            "0.7D54", "0.7D53", "0.7D52", "0.7D51", "0.7D50", "0.7D48", "0.7D47", "0.7D46",
            "0.7D45", "0.7D44", "0.7D43", "0.7D42", "0.7D41", "0.7D40", "0.7D39", "0.7D38",
            "0.7D37", "0.7D36", "0.7D35", "0.7D34", "0.7D33", "0.7D32", "0.7D31", "0.7D30",
            "0.7D29", "0.7D28", "0.7D27", "0.7D26", "0.7D25", "0.7D24", "0.7D21", "0.7D20",
            "0.7D19", "0.7D18", "0.7D17", "0.7D16", "0.7D15", "0.7D14", "0.7D13", "0.7D12",
            "0.7D11", "0.7D10", "0.7D9", "0.7D8", "0.7D7", "0.7D6", "0.7D5", "0.7D4", "0.7D",
            "0.7C6", "0.7C5", "0.7C4", "0.7C3", "0.7C2", "0.7C", "0.7B12", "0.7B11", "0.7B10",
            "0.7B8", "0.7B7", "0.7B6", "0.7B5", "0.7B3", "0.7B2", "0.7B", "0.7A13", "0.7A12",
            "0.7A11", "0.7A10", "0.7A9", "0.7A7", "0.7A6", "0.7A5", "0.7A3", "0.7A2", "0.7A",
            "0.6W60", "0.6W59", "0.6W58", "0.6W57", "0.6W56", "0.6W55", "0.6W54", "0.6W53",
            "0.6W52", "0.7F", "0.6W51", "0.6W50", "0.6W49", "0.6W48", "0.6W47", "0.6W46", "0.6W45",
            "0.6W43", "0.6V3", "0.6V", "0.6U25", "0.6U24", "0.6U23", "0.6U22", "0.6U21", "0.6U20",
            "0.6U19", "0.6U18", "0.6U17", "0.6U16", "0.6U15", "0.6U14", "0.6U13", "0.6U12",
            "0.6U11", "0.6U9", "0.6U7", "0.6U6", "0.6U5", "0.7F", "0.6U4", "0.6U3", "0.6U2",
            "0.6U", "0.6T7", "0.6T6", "0.6T5", "0.6T4", "0.6T3", "0.6T2", "0.6T", "0.6R22",
            "0.6R21", "0.6R20", "0.6R19", "0.6R18", "0.6R17", "0.6R16", "0.6R15", "0.6R14",
            "0.6R13", "0.6R12", "0.6R11", "0.6R9", "0.6R8", "0.6R7", "0.6R", "0.6Q14", "0.6Q12",
            "0.6Q10", "0.6Q9", "0.6Q3", "0.6Q", "0.6P9", "0.6P8", "0.6P7", "0.6P6", "0.6P5",
            "0.6P4", "0.6P3", "0.6P2", "0.6P", "0.6N7", "0.6N6", "0.6N4", "0.6N3", "0.6N2", "0.6N",
            "0.6M9", "0.6M8", "0.6M7", "0.6M6", "0.6M5", "0.6M3", "0.6M2", "0.6M", "0.7F",
            "0.6K26", "0.6K25", "0.6K24", "0.6K23", "0.6K22", "0.6K21", "0.6K20", "0.6K19",
            "0.6K18", "0.6K17", "0.6K16", "0.6K14", "0.6K12", "0.6K11", "0.6K10", "0.6K9", "0.6K8",
            "0.6K7", "0.6K6", "0.6K5", "0.6K4", "0.6K3", "0.6K2", "0.6K", "0.6J5", "0.6J4",
            "0.6J3", "0.6J2", "0.6J", "0.7F", "0.6H10", "0.6H6", "0.6H5", "0.6H4", "0.6H3",
            "0.6H2", "0.6H", "0.6G19", "0.6G18", "0.6G17", "0.6G16", "0.6G14", "0.6G3", "0.6G2",
            "0.6G", "0.6F12", "0.6F11", "0.6F10", "0.6F9", "0.6F8", "0.6F7", "0.6F6", "0.6F5",
            "0.6F4", "0.6F3", "0.6F2", "0.6F", "0.6E19", "0.6E18", "0.7F", "0.6E17", "0.6E16",
            "0.6E15", "0.6E14", "0.6E13", "0.6E12", "0.6E11", "0.6E10", "0.6E8", "0.6E7", "0.6E6",
            "0.6E5", "0.6E4", "0.6E", "0.6B16", "0.6B15", "0.6B14", "0.6B13", "0.6B12", "0.6B11",
            "0.6B10", "0.6B9", "0.6B8", "0.6B7", "0.6B6", "0.6B5", "0.6B", "0.6A4", "0.6A3",
            "0.7F", "0.6A2", "0.6A1", "0.5Z34", "0.5Z33", "0.5Z32", "0.5Z31", "0.5Z30", "0.5Z28",
            "0.5Z27", "0.5Z26", "0.5Z25", "0.5Z24", "0.5Z22", "0.5Z20", "0.5Z19", "0.5Z18",
            "0.5Z17", "0.5Z16", "0.5Z15", "0.5Z13", "0.5Z10", "0.5Z9", "0.5Z8", "0.5Z7", "0.5Z6",
            "0.5Z5", "0.5Z4", "0.5Z3", "0.5Z", "0.5Y32", "0.5Y31", "0.5Y30", "0.5Y24", "0.5Y22",
            "0.5Y21", "0.5Y20", "0.5Y19", "0.5Y18", "0.5Y16", "0.5Y15", "0.5Y14", "0.5Y13",
            "0.5Y12", "0.5Y11", "0.5Y10", "0.5Y9", "0.5Y8", "0.5Y", "0.5X39", "0.5X38", "0.5X37",
            "0.5X36", "0.5X35", "0.5X33", "0.5X32", "0.5X31", "0.5X30", "0.5X10", "0.5X8", "0.5X7",
            "0.5X6", "0.5X5", "0.5X4", "0.5X3", "0.5X2", "0.5X", "0.5W48", "0.5W47", "0.5W44",
            "0.5W43", "0.5W42", "0.5W41", "0.5W40", "0.5W39", "0.5W38", "0.5W37", "0.5W36",
            "0.5W35", "0.5W34", "0.5W33", "0.5W32", "0.5W26", "0.5W25", "0.5W24", "0.5W20",
            "0.5W17", "0.5W10", "0.5W9", "0.5W", "0.5V9", "0.5V5", "0.5V3", "0.5V2", "0.5V",
            "0.5U35", "0.5U34", "0.5U33", "0.5U32", "0.5U30", "0.5U10", "0.5U9", "0.5U7", "0.5U",
            "0.5T7", "0.5T6", "0.5T5", "0.5T4", "0.5T3", "0.5T2", "0.5T", "0.5S", "0.5Q", "0.5P12",
            "0.5P5", "0.5P4", "0.5P3", "0.5P2", "0.5P", "0.5L", "0.5K", "0.3H6", "0.3H5", "0.3H4",
            "0.3H3", "0.3H2", "0.3H", "0.3G10", "0.3G9", "0.3G8", "0.3G7", "0.3G6", "0.3G5",
            "0.3G4", "0.3G3", "0.3G", "0.3F", "0.3E12", "0.3E10", "0.3E8", "0.3E7", "0.3E6",
            "0.3E5", "0.3E4", "0.3E", "0.3D", "0.3C", "0.3B", "0.3A", "0.2F", "0.2E5", "0.2E4",
            "0.2E1", "0.2D4", "0.2D3", "0.2D2", "0.2D", "0.2C", "0.2B", "0.2A", "0.1W", "0.1T",
            "0.1Q", "0.1P", "0.1N", "0.1M", "0.1L", "0.1K", "0.1J", "0.1H3", "0.1H2", "0.1H",
            "0.1G3", "0.1G2", "0.1G", "0.1F2", "0.1F", "0.1E", "0.1D", "0.1C", "0.1B", "0.04Q",
            "0.04K",
        ];

        for i in version.iter() {
            let parsed = GameVersion::from_str(&i).unwrap();
            assert_eq!(i, &parsed.to_string());
        }
    }

    #[test]
    fn test_ordering() {
        assert!(GameVersion::from_str("0.04k").unwrap() < GameVersion::from_str("0.1").unwrap());
        assert!(GameVersion::from_str("0.1").unwrap() < GameVersion::from_str("0.1P").unwrap());
        assert!(GameVersion::from_str("0.3D").unwrap() < GameVersion::from_str("0.3e").unwrap());
        assert!(GameVersion::from_str("0.7F").unwrap() < GameVersion::from_str("0.7F1").unwrap());
        assert!(GameVersion::from_str("0.7F").unwrap() < GameVersion::from_str("0.7F1").unwrap());
        assert!(GameVersion::from_str("0.7F").unwrap() < GameVersion::from_str("0.8").unwrap());

        assert!(GameVersion::from_str("0.7F").unwrap() == GameVersion::from_str("0.7f").unwrap());
    }

    #[test]
    fn test_parse() {
        let res = "0.04k".parse::<GameVersion>();
        assert!(res.is_ok());
    }

    #[test]
    fn test_failure_to_parse_major() {
        let res = "a4k".parse::<GameVersion>();
        assert!(matches!(res, Err(GameVersionParseError::Major(_))));
    }

    #[test]
    fn test_failure_to_parse_minor() {
        let res = "0.04-".parse::<GameVersion>();
        assert!(matches!(res, Err(GameVersionParseError::Minor(_))));
    }

    #[test]
    fn test_failure_to_parse_patch() {
        let res = "0.04k-".parse::<GameVersion>();
        assert!(matches!(res, Err(GameVersionParseError::Patch(_))));
    }

    #[test]
    fn test_normalise_to_uppercase() {
        let res = "0.04k".parse::<GameVersion>().unwrap();
        assert!(res.minor == 'K');
    }

    #[test]
    fn test_from_to_bytes() {
        let ver = GameVersion::from_str("0.7F").unwrap();
        let mut buf = BytesMut::new();
        assert!(ver.encode(&mut buf).is_ok());

        assert_eq!(buf.len(), 8);

        let from = GameVersion::decode(&mut buf.freeze()).unwrap();

        assert_eq!(ver, from);
    }
}
