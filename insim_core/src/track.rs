//! Strongly typed Tracks

use std::str::FromStr;

use crate::{Decode, Encode, license::License};

macro_rules! define_tracks {
    (
        $(
            $variant:ident, // Enum Variant (e.g., Bl1)
            $code:literal,  // String Code (e.g., "BL1")
            $name:literal,  // Full Name (e.g., "Blackwood GP")
            $license:ident, // License Enum (e.g., Demo)
            $dist:expr      // Distance Miles (e.g., Some(2.0))
        ),* $(,)?
    ) => {
        #[derive(Debug, PartialEq, Eq, Copy, Clone, Hash, Default)]
        /// Handles parsing a Track name.
        #[non_exhaustive]
        #[allow(missing_docs)]
        #[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
        #[cfg_attr(feature = "serde", serde(try_from = "String", into = "String"))]
        pub enum Track {
            #[default]
            $($variant),*
        }

        impl Track {
            /// What license is required to access this map
            pub fn license(&self) -> License {
                match self {
                    $(Self::$variant => License::$license),*
                }
            }

            /// Driving distance in miles
            pub fn distance_mile(&self) -> Option<f32> {
                match self {
                    $(Self::$variant => $dist),*
                }
            }

            /// Driving distance in kilometers
            pub fn distance_km(&self) -> Option<f32> {
                self.distance_mile().map(|distance| distance * 1.60934)
            }

            /// Complete name of the track
            pub fn complete_name(&self) -> &'static str {
                match self {
                    $(Self::$variant => $name),*
                }
            }

            /// Track short code
            pub fn code(&self) -> &'static str {
                match self {
                    $(Self::$variant => $code),*
                }
            }

            /// Is this a reversed track?
            pub fn is_reverse(&self) -> bool {
                let s = self.code();
                s.ends_with('R') || s.ends_with('Y')
            }

            /// Is this an open world track?
            pub fn is_open(&self) -> bool {
                let s = self.code();
                s.ends_with('X') || s.ends_with('Y')
            }
        }

        impl FromStr for Track {
            type Err = TrackUnknownError;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    $($code => Ok(Self::$variant)),*,
                    _ => Err(TrackUnknownError(s.to_string())),
                }
            }
        }
    };
}

#[rustfmt::skip]
define_tracks!(
    // Variant, Code, Name,                            License, Distance (Miles)
    Bl1,   "BL1",   "Blackwood GP Track",              Demo, Some(2.0),
    Bl1r,  "BL1R",  "Blackwood GP Track R",            Demo, Some(2.0),
    Bl1x,  "BL1X",  "Blackwood GP Track X",            Demo, None,
    Bl1y,  "BL1Y",  "Blackwood GP Track Y",            Demo, None,
    Bl2,   "BL2",   "Blackwood Historic",              Demo, Some(2.0),
    Bl2r,  "BL2R",  "Blackwood Historic R",            Demo, Some(2.0),
    Bl2x,  "BL2X",  "Blackwood Historic X",            Demo, None,
    Bl2y,  "BL2Y",  "Blackwood Historic Y",            Demo, None,
    Bl3,   "BL3",   "Blackwood Rallycross",            Demo, Some(2.0),
    Bl3r,  "BL3R",  "Blackwood Rallycross R",          Demo, Some(2.0),
    Bl3x,  "BL3X",  "Blackwood Rallycross X",          Demo, None,
    Bl3y,  "BL3Y",  "Blackwood Rallycross Y",          Demo, None,
    Bl4,   "BL4",   "Blackwood Carpark",               Demo, None,
    Bl4x,  "BL4X",  "Blackwood Carpark X",             Demo, None,

    So1,   "SO1",   "South City Classic",              S1,   Some(1.3),
    So1r,  "SO1R",  "South City Classic R",            S1,   Some(1.3),
    So1x,  "SO1X",  "South City Classic X",            S1,   None,
    So1y,  "SO1Y",  "South City Classic Y",            S1,   None,
    So2,   "SO2",   "South City Sprint 1",             S1,   Some(1.3),
    So2r,  "SO2R",  "South City Sprint 1 R",           S1,   Some(1.3),
    So2x,  "SO2X",  "South City Sprint 1 X",           S1,   None,
    So2y,  "SO2Y",  "South City Sprint 1 Y",           S1,   None,
    So3,   "SO3",   "South City Sprint 2",             S1,   Some(0.8),
    So3r,  "SO3R",  "South City Sprint 2 R",           S1,   Some(0.8),
    So3x,  "SO3X",  "South City Sprint 2 X",           S1,   None,
    So3y,  "SO3Y",  "South City Sprint 2 Y",           S1,   None,
    So4,   "SO4",   "South City City Long",            S1,   Some(2.5),
    So4r,  "SO4R",  "South City City Long R",          S1,   Some(2.5),
    So4x,  "SO4X",  "South City City Long X",          S1,   None,
    So4y,  "SO4Y",  "South City City Long Y",          S1,   None,
    So5,   "SO5",   "South City Town Course",          S1,   Some(2.0),
    So5r,  "SO5R",  "South City Town Course R",        S1,   Some(2.0),
    So5x,  "SO5X",  "South City Town Course X",        S1,   None,
    So5y,  "SO5Y",  "South City Town Course Y",        S1,   None,
    So6,   "SO6",   "South City Chicane Course",       S1,   Some(1.8),
    So6r,  "SO6R",  "South City Chicane Course R",     S1,   Some(1.8),
    So6x,  "SO6X",  "South City Chicane Course X",     S1,   None,
    So6y,  "SO6Y",  "South City Chicane Course Y",     S1,   None,

    Fe1,   "FE1",   "Fern Bay Club",                   S1,   Some(1.0),
    Fe1r,  "FE1R",  "Fern Bay Club R",                 S1,   Some(1.0),
    Fe1x,  "FE1X",  "Fern Bay Club X",                 S1,   None,
    Fe1y,  "FE1Y",  "Fern Bay Club Y",                 S1,   None,
    Fe2,   "FE2",   "Fern Bay Green",                  S1,   Some(1.9),
    Fe2r,  "FE2R",  "Fern Bay Green R",                S1,   Some(1.9),
    Fe2x,  "FE2X",  "Fern Bay Green X",                S1,   None,
    Fe2y,  "FE2Y",  "Fern Bay Green Y",                S1,   None,
    Fe3,   "FE3",   "Fern Bay Gold",                   S1,   Some(2.2),
    Fe3r,  "FE3R",  "Fern Bay Gold R",                 S1,   Some(2.2),
    Fe3x,  "FE3X",  "Fern Bay Gold X",                 S1,   None,
    Fe3y,  "FE3Y",  "Fern Bay Gold Y",                 S1,   None,
    Fe4,   "FE4",   "Fern Bay Black",                  S1,   Some(4.1),
    Fe4r,  "FE4R",  "Fern Bay Black R",                S1,   Some(4.1),
    Fe4x,  "FE4X",  "Fern Bay Black X",                S1,   None,
    Fe4y,  "FE4Y",  "Fern Bay Black Y",                S1,   None,
    Fe5,   "FE5",   "Fern Bay Rallycross",             S1,   Some(1.3),
    Fe5r,  "FE5R",  "Fern Bay Rallycross R",           S1,   Some(1.3),
    Fe5x,  "FE5X",  "Fern Bay Rallycross X",           S1,   None,
    Fe5y,  "FE5Y",  "Fern Bay Rallycross Y",           S1,   None,
    Fe6,   "FE6",   "Fern Bay Rallycross Green",       S1,   Some(0.5),
    Fe6r,  "FE6R",  "Fern Bay Rallycross Green R",     S1,   Some(0.5),
    Fe6x,  "FE6X",  "Fern Bay Rallycross Green X",     S1,   None,
    Fe6y,  "FE6Y",  "Fern Bay Rallycross Green Y",     S1,   None,

    Au1,   "AU1",   "Autocross",                       S1,   None,
    Au1x,  "AU1X",  "Autocross X",                     S1,   None,
    Au2,   "AU2",   "Skid Pad",                        S1,   None,
    Au2x,  "AU2X",  "Skid Pad X",                      S1,   None,
    Au3,   "AU3",   "Drag Strip",                      S1,   None,
    Au3x,  "AU3X",  "Drag Strip X",                    S1,   None,
    Au4,   "AU4",   "8 Lane Drag Strip",               S1,   None,
    Au4x,  "AU4X",  "8 Lane Drag Strip X",             S1,   None,

    Ky1,   "KY1",   "Kyoto Oval",                      S2,   Some(1.9),
    Ky1r,  "KY1R",  "Kyoto Oval R",                    S2,   Some(1.9),
    Ky1x,  "KY1X",  "Kyoto Oval X",                    S2,   None,
    Ky1y,  "KY1Y",  "Kyoto Oval Y",                    S2,   None,
    Ky2,   "KY2",   "Kyoto National",                  S2,   Some(3.2),
    Ky2r,  "KY2R",  "Kyoto National R",                S2,   Some(3.2),
    Ky2x,  "KY2X",  "Kyoto National X",                S2,   None,
    Ky2y,  "KY2Y",  "Kyoto National Y",                S2,   None,
    Ky3,   "KY3",   "Kyoto GP Long",                   S2,   Some(4.6),
    Ky3r,  "KY3R",  "Kyoto GP Long R",                 S2,   Some(4.6),
    Ky3x,  "KY3X",  "Kyoto GP Long X",                 S2,   None,
    Ky3y,  "KY3Y",  "Kyoto GP Long Y",                 S2,   None,

    We1,   "WE1",   "Westhill National",               S2,   Some(2.7),
    We1r,  "WE1R",  "Westhill National R",             S2,   Some(2.7),
    We1x,  "WE1X",  "Westhill National X",             S2,   None,
    We1y,  "WE1Y",  "Westhill National Y",             S2,   None,
    We2,   "WE2",   "Westhill International",          S2,   Some(3.6),
    We2r,  "WE2R",  "Westhill International R",        S2,   Some(3.6),
    We2x,  "WE2X",  "Westhill International X",        S2,   None,
    We2y,  "WE2Y",  "Westhill International Y",        S2,   None,
    We3,   "WE3",   "Westhill Car Park",               S2,   None,
    We3x,  "WE3X",  "Westhill Car Park X",             S2,   None,
    We4,   "WE4",   "Westhill Karting",                S2,   Some(0.3),
    We4r,  "WE4R",  "Westhill Karting R",              S2,   Some(0.3),
    We4x,  "WE4X",  "Westhill Karting X",              S2,   None,
    We4y,  "WE4Y",  "Westhill Karting Y",              S2,   None,
    We5,   "WE5",   "Westhill Karting Long",           S2,   Some(0.8),
    We5r,  "WE5R",  "Westhill Karting Long R",         S2,   Some(0.8),
    We5x,  "WE5X",  "Westhill Karting Long X",         S2,   None,
    We5y,  "WE5Y",  "Westhill Karting Long Y",         S2,   None,

    As1,   "AS1",   "Aston Cadet",                     S2,   Some(1.2),
    As1r,  "AS1R",  "Aston Cadet R",                   S2,   Some(1.2),
    As1x,  "AS1X",  "Aston Cadet X",                   S2,   None,
    As1y,  "AS1Y",  "Aston Cadet Y",                   S2,   None,
    As2,   "AS2",   "Aston Club",                      S2,   Some(1.9),
    As2r,  "AS2R",  "Aston Club R",                    S2,   Some(1.9),
    As2x,  "AS2X",  "Aston Club X",                    S2,   None,
    As2y,  "AS2Y",  "Aston Club Y",                    S2,   None,
    As3,   "AS3",   "Aston National",                  S2,   Some(3.5),
    As3r,  "AS3R",  "Aston National R",                S2,   Some(3.5),
    As3x,  "AS3X",  "Aston National X",                S2,   None,
    As3y,  "AS3Y",  "Aston National Y",                S2,   None,
    As4,   "AS4",   "Aston Historic",                  S2,   Some(5.0),
    As4r,  "AS4R",  "Aston Historic R",                S2,   Some(5.0),
    As4x,  "AS4X",  "Aston Historic X",                S2,   None,
    As4y,  "AS4Y",  "Aston Historic Y",                S2,   None,
    As5,   "AS5",   "Aston Grand Prix",                S2,   Some(5.5),
    As5r,  "AS5R",  "Aston Grand Prix R",              S2,   Some(5.5),
    As5x,  "AS5X",  "Aston Grand Prix X",              S2,   None,
    As5y,  "AS5Y",  "Aston Grand Prix Y",              S2,   None,
    As6,   "AS6",   "Aston Grand Touring",             S2,   Some(5.0),
    As6r,  "AS6R",  "Aston Grand Touring R",           S2,   Some(5.0),
    As6x,  "AS6X",  "Aston Grand Touring X",           S2,   None,
    As6y,  "AS6Y",  "Aston Grand Touring Y",           S2,   None,
    As7,   "AS7",   "Aston North",                     S2,   Some(3.2),
    As7r,  "AS7R",  "Aston North R",                   S2,   Some(3.2),
    As7x,  "AS7X",  "Aston North X",                   S2,   None,
    As7y,  "AS7Y",  "Aston North Y",                   S2,   None,

    Ro1,   "RO1",   "Rockingham ISSC",                 S3,   Some(1.9),
    Ro1x,  "RO1X",  "Rockingham ISSC X",               S3,   None,
    Ro2,   "RO2",   "Rockingham National",             S3,   Some(1.7),
    Ro2x,  "RO2X",  "Rockingham National X",           S3,   None,
    Ro3,   "RO3",   "Rockingham Oval",                 S3,   Some(1.5),
    Ro3x,  "RO3X",  "Rockingham Oval X",               S3,   None,
    Ro4,   "RO4",   "Rockingham ISSC Long",            S3,   Some(2.0),
    Ro4x,  "RO4X",  "Rockingham ISSC Long X",          S3,   None,
    Ro5,   "RO5",   "Rockingham Lake",                 S3,   Some(0.7),
    Ro5x,  "RO5X",  "Rockingham Lake X",               S3,   None,
    Ro6,   "RO6",   "Rockingham Handling",             S3,   Some(1.0),
    Ro6x,  "RO6X",  "Rockingham Handling X",           S3,   None,
    Ro7,   "RO7",   "Rockingham International",        S3,   Some(2.4),
    Ro7x,  "RO7X",  "Rockingham International X",      S3,   None,
    Ro8,   "RO8",   "Rockingham Historic",             S3,   Some(2.2),
    Ro8x,  "RO8X",  "Rockingham Historic X",           S3,   None,
    Ro9,   "RO9",   "Rockingham Historic Short",       S3,   Some(1.4),
    Ro9x,  "RO9X",  "Rockingham Historic Short X",     S3,   None,
    Ro10,  "RO10",  "Rockingham International Long",   S3,   Some(2.5),
    Ro10x, "RO10X", "Rockingham International Long X", S3, None,
    Ro11,  "RO11",  "Rockingham Sportscar",            S3,   Some(1.7),
    Ro11x, "RO11X", "Rockingham Sportscar X",          S3,   None,

    La1,   "LA1",   "Layout Square Long Grid",         S3,   None,
    La1x,  "LA1X",  "Layout Square Long Grid X",       S3,   None,
    La2,   "LA2",   "Layout Square Wide Grid",         S3,   None,
    La2x,  "LA2X",  "Layout Square Wide Grid X",       S3,   None
);

impl Decode for Track {
    fn decode(buf: &mut bytes::Bytes) -> Result<Self, crate::DecodeError> {
        let raw = buf.split_to(6);
        let s = std::str::from_utf8(&raw).unwrap_or("").trim_matches('\0');
        s.parse().map_err(|_| crate::DecodeError::BadMagic {
            found: Box::new(raw),
        })
    }
}

impl Encode for Track {
    fn encode(&self, buf: &mut bytes::BytesMut) -> Result<(), crate::EncodeError> {
        let s = self.code();
        let bytes = s.as_bytes();

        if bytes.len() > 6 {
            return Err(crate::EncodeError::TooLarge);
        }
        buf.extend_from_slice(bytes);
        buf.resize(buf.len() + (6 - bytes.len()), 0);
        Ok(())
    }
}

impl std::fmt::Display for Track {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.code())
    }
}

#[derive(Debug, thiserror::Error)]
/// Unknown Track Error
pub struct TrackUnknownError(String);

impl std::fmt::Display for TrackUnknownError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<String> for Track {
    type Error = <Track as FromStr>::Err;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        s.parse()
    }
}

impl From<Track> for String {
    fn from(track: Track) -> Self {
        track.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bl1x_from_str() {
        let v = Track::from_str("BL1X").expect("Expected to handle BL1X");
        assert_eq!(v, Track::Bl1x);
        assert_eq!("BL1X", v.to_string());
    }

    #[test]
    fn test_unknown_from_str() {
        let v = Track::from_str("");
        assert!(matches!(v, Err(TrackUnknownError(_))));
    }

    #[test]
    fn test_reverse_logic() {
        assert!(Track::Bl1r.is_reverse());
        assert!(!Track::Bl1.is_reverse());
        assert!(!Track::Bl1.is_open());
        assert!(Track::Bl1y.is_reverse());
        assert!(Track::Bl1y.is_open());
    }
}
