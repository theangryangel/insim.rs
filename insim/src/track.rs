//! Utility functions and structs for working with track names and fetching track data.

use bytes::BytesMut;
use insim_core::{prelude::*, ser::Limit, DecodableError, EncodableError};

#[cfg(feature = "serde")]
use serde::Serialize;

use std::collections::HashMap;

use insim_core::string::strip_trailing_nul;
use once_cell::sync::Lazy;

#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct TrackInfo {
    pub code: String,
    pub name: String,
    pub distance: Option<f32>,
    pub max_players: u8,
}

impl TrackInfo {
    pub fn family(&self) -> String {
        self.code.chars().take(2).collect()
    }
}

pub static TRACK_INFO: Lazy<HashMap<String, TrackInfo>> = Lazy::new(|| {
    let mut m = HashMap::new();

    m.insert(
        "BL1".into(),
        TrackInfo {
            code: "BL1".into(),
            name: "Blackwood GP".into(),
            max_players: 40,
            distance: Some(2.048),
        },
    );
    m.insert(
        "BL2".into(),
        TrackInfo {
            code: "BL2".into(),
            name: "Blackwood Historic".into(),
            max_players: 40,
            distance: Some(2.047),
        },
    );
    m.insert(
        "BL3".into(),
        TrackInfo {
            code: "BL3".into(),
            name: "Blackwood RallyX".into(),
            max_players: 40,
            distance: Some(1.142),
        },
    );
    m.insert(
        "BL4".into(),
        TrackInfo {
            code: "BL4".into(),
            name: "Blackwood Carpark".into(),
            max_players: 40,
            distance: None,
        },
    );
    m.insert(
        "SO1".into(),
        TrackInfo {
            code: "SO1".into(),
            name: "South City Classic".into(),
            max_players: 30,
            distance: Some(1.263),
        },
    );
    m.insert(
        "SO2".into(),
        TrackInfo {
            code: "SO2".into(),
            name: "South City Sprint 1".into(),
            max_players: 16,
            distance: Some(1.273),
        },
    );
    m.insert(
        "SO3".into(),
        TrackInfo {
            code: "SO3".into(),
            name: "South City Sprint 2".into(),
            max_players: 16,
            distance: Some(0.829),
        },
    );
    m.insert(
        "SO4".into(),
        TrackInfo {
            code: "SO4".into(),
            name: "South City City".into(),
            max_players: 32,
            distance: Some(2.504),
        },
    );
    m.insert(
        "SO5".into(),
        TrackInfo {
            code: "SO5".into(),
            name: "South City Town".into(),
            max_players: 32,
            distance: Some(1.955),
        },
    );
    m.insert(
        "SO6".into(),
        TrackInfo {
            code: "SO6".into(),
            name: "South City Chicane".into(),
            max_players: 32,
            distance: Some(1.813),
        },
    );
    m.insert(
        "FE1".into(),
        TrackInfo {
            code: "FE1".into(),
            name: "Fern Bay Club".into(),
            max_players: 32,
            distance: Some(0.984),
        },
    );
    m.insert(
        "FE2".into(),
        TrackInfo {
            code: "FE2".into(),
            name: "Fern Bay Green".into(),
            max_players: 32,
            distance: Some(1.918),
        },
    );
    m.insert(
        "FE3".into(),
        TrackInfo {
            code: "FE3".into(),
            name: "Fern Bay Gold".into(),
            max_players: 32,
            distance: Some(2.183),
        },
    );
    m.insert(
        "FE4".into(),
        TrackInfo {
            code: "FE4".into(),
            name: "Fern Bay Black".into(),
            max_players: 32,
            distance: Some(4.076),
        },
    );
    m.insert(
        "FE5".into(),
        TrackInfo {
            code: "FE5".into(),
            name: "Fern Bay RallyX".into(),
            max_players: 32,
            distance: Some(1.254),
        },
    );
    m.insert(
        "FE6".into(),
        TrackInfo {
            code: "FE6".into(),
            name: "Fern Bay RallyX Green".into(),
            max_players: 32,
            distance: Some(0.463),
        },
    );
    m.insert(
        "AU1".into(),
        TrackInfo {
            code: "AU1".into(),
            name: "AutoX".into(),
            max_players: 16,
            distance: None,
        },
    );
    m.insert(
        "AU2".into(),
        TrackInfo {
            code: "AU2".into(),
            name: "Skidpad".into(),
            max_players: 16,
            distance: None,
        },
    );
    m.insert(
        "AU3".into(),
        TrackInfo {
            code: "AU3".into(),
            name: "2 Drag Strip".into(),
            max_players: 2,
            distance: Some(0.250),
        },
    );
    m.insert(
        "AU4".into(),
        TrackInfo {
            code: "AU4".into(),
            name: "8 Lane Drag Strip".into(),
            max_players: 8,
            distance: Some(0.250),
        },
    );
    m.insert(
        "KY1".into(),
        TrackInfo {
            code: "KY1".into(),
            name: "Kyoto Ring Oval".into(),
            max_players: 32,
            distance: Some(1.852),
        },
    );
    m.insert(
        "KY2".into(),
        TrackInfo {
            code: "KY2".into(),
            name: "Kyoto Ring National".into(),
            max_players: 32,
            distance: Some(3.193),
        },
    );
    m.insert(
        "KY3".into(),
        TrackInfo {
            code: "KY3".into(),
            name: "Kyoto Ring GP Long".into(),
            max_players: 32,
            distance: Some(4.584),
        },
    );
    m.insert(
        "WE1".into(),
        TrackInfo {
            code: "WE1".into(),
            name: "Westhill National".into(),
            max_players: 40,
            distance: Some(2.732),
        },
    );
    m.insert(
        "WE2".into(),
        TrackInfo {
            code: "WE2".into(),
            name: "Westhill International".into(),
            max_players: 40,
            distance: Some(3.573),
        },
    );
    m.insert(
        "WE3".into(),
        TrackInfo {
            code: "WE3".into(),
            name: "Westhill Car Park".into(),
            max_players: 40,
            distance: None,
        },
    );
    m.insert(
        "WE4".into(),
        TrackInfo {
            code: "WE4".into(),
            name: "Westhill Karting".into(),
            max_players: 40,
            distance: Some(0.3),
        },
    );
    m.insert(
        "WE5".into(),
        TrackInfo {
            code: "WE5".into(),
            name: "Westhill Karting National".into(),
            max_players: 40,
            distance: Some(0.818),
        },
    );
    m.insert(
        "AS1".into(),
        TrackInfo {
            code: "AS1".into(),
            name: "Aston Cadet".into(),
            max_players: 32,
            distance: Some(1.162),
        },
    );
    m.insert(
        "AS2".into(),
        TrackInfo {
            code: "AS2".into(),
            name: "Aston Club".into(),
            max_players: 32,
            distance: Some(1.912),
        },
    );
    m.insert(
        "AS3".into(),
        TrackInfo {
            code: "AS3".into(),
            name: "Aston National".into(),
            max_players: 32,
            distance: Some(3.481),
        },
    );
    m.insert(
        "AS4".into(),
        TrackInfo {
            code: "AS4".into(),
            name: "Aston Historic".into(),
            max_players: 32,
            distance: Some(5.026),
        },
    );
    m.insert(
        "AS5".into(),
        TrackInfo {
            code: "AS5".into(),
            name: "Aston GP".into(),
            max_players: 32,
            distance: Some(5.469),
        },
    );
    m.insert(
        "AS6".into(),
        TrackInfo {
            code: "AS6".into(),
            name: "Aston Grand Touring".into(),
            max_players: 32,
            distance: Some(4.972),
        },
    );
    m.insert(
        "AS7".into(),
        TrackInfo {
            code: "AS7".into(),
            name: "Aston North".into(),
            max_players: 32,
            distance: Some(3.211),
        },
    );
    m.insert(
        "RO1".into(),
        TrackInfo {
            code: "RO1".into(),
            name: "Rockingham ISSC".into(),
            max_players: 40,
            distance: Some(1.924),
        },
    );
    m.insert(
        "RO2".into(),
        TrackInfo {
            code: "RO2".into(),
            name: "Rockingham National".into(),
            max_players: 40,
            distance: Some(1.676),
        },
    );
    m.insert(
        "RO3".into(),
        TrackInfo {
            code: "RO3".into(),
            name: "Rockingham Oval".into(),
            max_players: 40,
            distance: Some(1.468),
        },
    );
    m.insert(
        "RO4".into(),
        TrackInfo {
            code: "RO4".into(),
            name: "Rockingham ISSC Long".into(),
            max_players: 40,
            distance: Some(2.021),
        },
    );
    m.insert(
        "RO5".into(),
        TrackInfo {
            code: "RO5".into(),
            name: "Rockingham Lake".into(),
            max_players: 40,
            distance: Some(0.650),
        },
    );
    m.insert(
        "RO6".into(),
        TrackInfo {
            code: "RO6".into(),
            name: "Rockingham Handling".into(),
            max_players: 40,
            distance: Some(1.559),
        },
    );
    m.insert(
        "RO7".into(),
        TrackInfo {
            code: "RO7".into(),
            name: "Rockingham International".into(),
            max_players: 40,
            distance: Some(2.407),
        },
    );
    m.insert(
        "RO8".into(),
        TrackInfo {
            code: "RO8".into(),
            name: "Rockingham Historic".into(),
            max_players: 40,
            distance: Some(2.215),
        },
    );
    m.insert(
        "RO9".into(),
        TrackInfo {
            code: "RO9".into(),
            name: "Rockingham Historic Short".into(),
            max_players: 40,
            distance: Some(1.365),
        },
    );
    m.insert(
        "RO10".into(),
        TrackInfo {
            code: "RO10".into(),
            name: "Rockingham International Long".into(),
            max_players: 40,
            distance: Some(2.521),
        },
    );
    m.insert(
        "RO11".into(),
        TrackInfo {
            code: "RO11".into(),
            name: "Rockingham Sports Car".into(),
            max_players: 40,
            distance: Some(1.674),
        },
    );
    m.insert(
        "LA1".into(),
        TrackInfo {
            code: "LA1".into(),
            name: "Layout Square, Long Grid".into(),
            max_players: 40,
            distance: None,
        },
    );
    m.insert(
        "LA2".into(),
        TrackInfo {
            code: "LA2".into(),
            name: "Layout Square, Wide Grid".into(),
            max_players: 40,
            distance: None,
        },
    );

    m
});

/// Lookup a [TrackInfo] from a track name
pub fn lookup(input: &[u8]) -> Option<TrackInfo> {
    if let Some(rpos) = input.iter().rposition(|x| ![b'X', b'R', 0].contains(x)) {
        if let Ok(short_code) = std::str::from_utf8(&input[..=rpos]) {
            return TRACK_INFO.get(short_code).cloned();
        }
    }
    None
}

/// Handles parsing a Track name.
#[derive(Debug, PartialEq, Eq, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct Track {
    pub inner: [u8; 6],
}

impl Track {
    /// Is this a reversed track?
    pub fn is_reverse(&self) -> bool {
        matches!(strip_trailing_nul(&self.inner).last(), Some(b'R'))
    }

    /// Are we in open world mode?
    pub fn is_open_world(&self) -> bool {
        matches!(strip_trailing_nul(&self.inner).last(), Some(b'X'))
    }

    /// Lookup the [TrackInfo] for this track.
    pub fn track_info(&self) -> Option<TrackInfo> {
        lookup(&self.inner)
    }
}

impl std::fmt::Display for Track {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let stripped = strip_trailing_nul(&self.inner);
        write!(f, "{}", String::from_utf8_lossy(stripped))
    }
}

impl Encodable for Track {
    fn encode(
        &self,
        buf: &mut bytes::BytesMut,
        limit: Option<Limit>,
    ) -> Result<(), EncodableError> {
        if let Some(limit) = limit {
            return Err(EncodableError::UnexpectedLimit(format!(
                "Track does not support limit: {limit:?}",
            )));
        }

        for i in self.inner.iter() {
            i.encode(buf, None)?;
        }

        Ok(())
    }
}

impl Decodable for Track {
    fn decode(buf: &mut BytesMut, limit: Option<Limit>) -> Result<Self, DecodableError> {
        if let Some(limit) = limit {
            return Err(DecodableError::UnexpectedLimit(format!(
                "Track does not support limit: {limit:?}",
            )));
        }

        let mut data: Track = Default::default();
        for i in 0..6 {
            data.inner[i] = u8::decode(buf, None)?;
        }

        Ok(data)
    }
}
