//! Utility functions and structs for working with track names and fetching track data.

use crate::string::strip_trailing_nul;
use deku::prelude::*;
#[cfg(feature = "serde")]
use serde::Serialize;

pub type TrackInfo<'a> = (&'a str, &'a str, Option<f32>, u8);

const TRACK_INFO: &[TrackInfo] = &[
    ("BL1", "Blackwood GP", Some(2.048), 40),
    ("BL2", "Blackwood Historic", Some(2.047), 40),
    ("BL3", "Blackwood RallyX", Some(1.142), 40),
    ("BL4", "Blackwood Carpark", None, 40),
    ("SO1", "South City Classic", Some(1.263), 30),
    ("SO2", "South City Sprint 1", Some(1.273), 16),
    ("SO3", "South City Sprint 2", Some(0.829), 16),
    ("SO4", "South City City", Some(2.504), 32),
    ("SO5", "South City Town", Some(1.955), 32),
    ("SO6", "South City Chicane", Some(1.813), 32),
    ("FE1", "Fern Bay Club", Some(0.984), 32),
    ("FE2", "Fern Bay Green", Some(1.918), 32),
    ("FE3", "Fern Bay Gold", Some(2.183), 32),
    ("FE4", "Fern Bay Black", Some(4.076), 32),
    ("FE5", "Fern Bay RallyX", Some(1.254), 32),
    ("FE6", "Fern Bay RallyX Green", Some(0.463), 32),
    ("AU1", "AutoX", None, 16),
    ("AU2", "Skidpad", None, 16),
    ("AU3", "2 Drag Strip", Some(0.250), 2),
    ("AU4", "8 Lane Drag Strip", Some(0.250), 8),
    ("KY1", "Kyoto Ring Oval", Some(1.852), 32),
    ("KY2", "Kyoto Ring National", Some(3.193), 32),
    ("KY3", "Kyoto Ring GP Long", Some(4.584), 32),
    ("WE1", "Westhill National", Some(2.732), 40),
    ("WE2", "Westhill International", Some(3.573), 40),
    ("WE3", "Westhill Car Park", None, 40),
    ("WE4", "Westhill Karting", Some(0.3), 40),
    ("WE5", "Westhill Karting National", Some(0.818), 40),
    ("AS1", "Aston Cadet", Some(1.162), 32),
    ("AS2", "Aston Club", Some(1.912), 32),
    ("AS3", "Aston National", Some(3.481), 32),
    ("AS4", "Aston Historic", Some(5.026), 32),
    ("AS5", "Aston GP", Some(5.469), 32),
    ("AS6", "Aston Grand Touring", Some(4.972), 32),
    ("AS7", "Aston North", Some(3.211), 32),
    ("RO1", "Rockingham ISSC", Some(1.924), 40),
    ("RO2", "Rockingham National", Some(1.676), 40),
    ("RO3", "Rockingham Oval", Some(1.468), 40),
    ("RO4", "Rockingham ISSC Long", Some(2.021), 40),
    ("RO5", "Rockingham Lake", Some(0.650), 40),
    ("RO6", "Rockingham Handling", Some(1.559), 40),
    ("RO7", "Rockingham International", Some(2.407), 40),
    ("RO8", "Rockingham Historic", Some(2.215), 40),
    ("RO9", "Rockingham Historic Short", Some(1.365), 40),
    ("RO10", "Rockingham International Long", Some(2.521), 40),
    ("RO11", "Rockingham Sports Car", Some(1.674), 40),
    ("LA1", "Layout Square, Long Grid", None, 0),
    ("LA2", "Layout Square, Wide Grid", None, 0),
];

/// Lookup a [TrackInfo] from a track name
pub fn lookup(input: &[u8]) -> Option<&TrackInfo> {
    if let Some(rpos) = input.iter().rposition(|x| ![b'X', b'R', 0].contains(x)) {
        if let Ok(short_code) = std::str::from_utf8(&input[..=rpos]) {
            return TRACK_INFO.iter().find(|x| x.0 == short_code);
        }
    }
    None
}

/// Handles parsing a Track name.
#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Default)]
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
    pub fn track_info(&self) -> Option<&TrackInfo> {
        lookup(&self.inner)
    }
}

impl std::fmt::Display for Track {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let stripped = strip_trailing_nul(&self.inner);
        write!(f, "{}", String::from_utf8_lossy(stripped))
    }
}
