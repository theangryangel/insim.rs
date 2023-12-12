//! Utility functions and structs for working with track names and fetching track data.
use insim_core::{license::License, track::Track};

pub trait TrackData: std::fmt::Display {
    fn required_license(&self) -> License;
    fn max_players(&self) -> u8;
    fn distance_km(&self) -> Option<f32>;

    fn full_name(&self) -> String;
    fn code(&self) -> String {
        self.to_string().to_uppercase()
    }
}

impl TrackData for Track {
    fn required_license(&self) -> License {
        match self {
            Self::Bl1 => License::Demo,
            Self::Bl1r => License::Demo,
            Self::Bl2 => License::Demo,
            Self::Bl2r => License::Demo,
            Self::Bl3 => License::Demo,
            Self::Bl3r => License::Demo,
            Self::Bl4 => License::Demo,
            Self::Blx => License::Demo,

            Self::So1 => License::S1,
            Self::So1r => License::S1,
            Self::So2 => License::S1,
            Self::So2r => License::S1,
            Self::So3 => License::S1,
            Self::So3r => License::S1,
            Self::So4 => License::S1,
            Self::So4r => License::S1,
            Self::So5 => License::S1,
            Self::So5r => License::S1,
            Self::So6 => License::S1,
            Self::So6r => License::S1,
            Self::Sox => License::S1,

            Self::Fe1 => License::S1,
            Self::Fe1r => License::S1,
            Self::Fe2 => License::S1,
            Self::Fe2r => License::S1,
            Self::Fe3 => License::S1,
            Self::Fe3r => License::S1,
            Self::Fe4 => License::S1,
            Self::Fe4r => License::S1,
            Self::Fe5 => License::S1,
            Self::Fe5r => License::S1,
            Self::Fe6 => License::S1,
            Self::Fe6r => License::S1,
            Self::Fex => License::S1,

            Self::Au1 => License::S1,
            Self::Au2 => License::S1,
            Self::Au3 => License::S1,
            Self::Au4 => License::S1,

            Self::Ky1 => License::S2,
            Self::Ky1r => License::S2,
            Self::Ky2 => License::S2,
            Self::Ky2r => License::S2,
            Self::Ky3 => License::S2,
            Self::Ky3r => License::S2,
            Self::Kyx => License::S2,

            Self::We1 => License::S2,
            Self::We1r => License::S2,
            Self::We2 => License::S2,
            Self::We2r => License::S2,
            Self::We3 => License::S2,
            Self::We4 => License::S2,
            Self::We4r => License::S2,
            Self::We5 => License::S2,
            Self::We5r => License::S2,
            Self::Wex => License::S2,

            Self::As1 => License::S2,
            Self::As1r => License::S2,
            Self::As2 => License::S2,
            Self::As2r => License::S2,
            Self::As3 => License::S2,
            Self::As3r => License::S2,
            Self::As4 => License::S2,
            Self::As4r => License::S2,
            Self::As5 => License::S2,
            Self::As5r => License::S2,
            Self::As6 => License::S2,
            Self::As6r => License::S2,
            Self::As7 => License::S2,
            Self::As7r => License::S2,
            Self::Asx => License::S2,

            Self::Ro1 => License::S3,
            Self::Ro2 => License::S3,
            Self::Ro3 => License::S3,
            Self::Ro4 => License::S3,
            Self::Ro5 => License::S3,
            Self::Ro6 => License::S3,
            Self::Ro7 => License::S3,
            Self::Ro8 => License::S3,
            Self::Ro9 => License::S3,
            Self::Ro10 => License::S3,
            Self::Ro11 => License::S3,
            Self::Rox => License::S3,

            Self::La1 => License::S3,
            Self::La2 => License::S3,

            _ => License::Demo,
        }
    }

    fn max_players(&self) -> u8 {
        match self {
            Self::Bl1 => 40,
            Self::Bl1r => 40,
            Self::Bl2 => 40,
            Self::Bl2r => 40,
            Self::Bl3 => 40,
            Self::Bl3r => 40,
            Self::Bl4 => 40,
            Self::Blx => 40,

            Self::So1 => 30,
            Self::So1r => 30,
            Self::So2 => 16,
            Self::So2r => 16,
            Self::So3 => 16,
            Self::So3r => 16,
            Self::So4 => 32,
            Self::So4r => 32,
            Self::So5 => 32,
            Self::So5r => 32,
            Self::So6 => 32,
            Self::So6r => 32,
            Self::Sox => 40,

            Self::Fe1 => 32,
            Self::Fe1r => 32,
            Self::Fe2 => 32,
            Self::Fe2r => 32,
            Self::Fe3 => 32,
            Self::Fe3r => 32,
            Self::Fe4 => 32,
            Self::Fe4r => 32,
            Self::Fe5 => 32,
            Self::Fe5r => 32,
            Self::Fe6 => 32,
            Self::Fe6r => 32,
            Self::Fex => 40,

            Self::Au1 => 16,
            Self::Au2 => 16,
            Self::Au3 => 2,
            Self::Au4 => 8,

            Self::Ky1 => 32,
            Self::Ky1r => 32,
            Self::Ky2 => 32,
            Self::Ky2r => 32,
            Self::Ky3 => 32,
            Self::Ky3r => 32,
            Self::Kyx => 40,

            Self::We1 => 40,
            Self::We1r => 40,
            Self::We2 => 40,
            Self::We2r => 40,
            Self::We3 => 40,
            Self::We4 => 40,
            Self::We4r => 40,
            Self::We5 => 40,
            Self::We5r => 40,
            Self::Wex => 40,

            Self::As1 => 32,
            Self::As1r => 32,
            Self::As2 => 32,
            Self::As2r => 32,
            Self::As3 => 32,
            Self::As3r => 32,
            Self::As4 => 32,
            Self::As4r => 32,
            Self::As5 => 32,
            Self::As5r => 32,
            Self::As6 => 32,
            Self::As6r => 32,
            Self::As7 => 32,
            Self::As7r => 32,
            Self::Asx => 40,

            Self::Ro1 => 40,
            Self::Ro2 => 40,
            Self::Ro3 => 40,
            Self::Ro4 => 40,
            Self::Ro5 => 40,
            Self::Ro6 => 40,
            Self::Ro7 => 40,
            Self::Ro8 => 40,
            Self::Ro9 => 40,
            Self::Ro10 => 40,
            Self::Ro11 => 40,
            Self::Rox => 40,

            Self::La1 => 40,
            Self::La2 => 40,

            _ => 0,
        }
    }

    fn distance_km(&self) -> Option<f32> {
        match self {
            Self::Bl1 | Self::Bl1r => Some(2.048),
            Self::Bl2 | Self::Bl2r => Some(2.047),
            Self::Bl3 | Self::Bl3r => Some(1.142),

            Self::So1 | Self::So1r => Some(1.263),
            Self::So2 | Self::So2r => Some(1.273),
            Self::So3 | Self::So3r => Some(0.829),
            Self::So4 | Self::So4r => Some(2.504),
            Self::So5 | Self::So5r => Some(1.955),
            Self::So6 | Self::So6r => Some(1.813),

            Self::Fe1 | Self::Fe1r => Some(0.984),
            Self::Fe2 | Self::Fe2r => Some(1.918),
            Self::Fe3 | Self::Fe3r => Some(2.183),
            Self::Fe4 | Self::Fe4r => Some(4.076),
            Self::Fe5 | Self::Fe5r => Some(1.254),
            Self::Fe6 | Self::Fe6r => Some(0.463),

            Self::Ky1 | Self::Ky1r => Some(1.852),
            Self::Ky2 | Self::Ky2r => Some(3.193),
            Self::Ky3 | Self::Ky3r => Some(4.584),

            Self::We1 | Self::We1r => Some(2.732),
            Self::We2 | Self::We2r => Some(3.573),
            Self::We4 | Self::We4r => Some(0.3),
            Self::We5 | Self::We5r => Some(0.818),

            Self::As1 | Self::As1r => Some(1.162),
            Self::As2 | Self::As2r => Some(1.912),
            Self::As3 | Self::As3r => Some(3.481),
            Self::As4 | Self::As4r => Some(5.026),
            Self::As5 | Self::As5r => Some(5.469),
            Self::As6 | Self::As6r => Some(4.972),
            Self::As7 | Self::As7r => Some(3.211),

            Self::Ro1 => Some(1.924),
            Self::Ro2 => Some(1.676),
            Self::Ro3 => Some(1.468),
            Self::Ro4 => Some(2.021),
            Self::Ro5 => Some(0.650),
            Self::Ro6 => Some(1.559),
            Self::Ro7 => Some(2.407),
            Self::Ro8 => Some(2.215),
            Self::Ro9 => Some(1.365),
            Self::Ro10 => Some(2.521),
            Self::Ro11 => Some(1.674),

            _ => None,
        }
    }

    fn full_name(&self) -> String {
        match self {
            Self::Bl1 => "Blackwood GP",
            Self::Bl1r => "Blackwood GP Reversed",
            Self::Bl2 => "Blackwood Historic",
            Self::Bl2r => "Blackwood Historic Reversed",
            Self::Bl3 => "Blackwood RallyX",
            Self::Bl3r => "Blackwood RallyX Reversed",
            Self::Bl4 => "Blackwood Carpark",
            Self::Blx => "Blackwood",

            Self::So1 => "South City Classic",
            Self::So1r => "South City Classic Reversed",
            Self::So2 => "South City Sprint 1",
            Self::So2r => "South City Sprint 1 Reversed",
            Self::So3 => "South City Sprint 2",
            Self::So3r => "South City Sprint 2 Reversed",
            Self::So4 => "South City Long",
            Self::So4r => "South City Long Reversed",
            Self::So5 => "South City Town",
            Self::So5r => "South City Town Reversed",
            Self::So6 => "South City Chicane",
            Self::So6r => "South City Chicane Reversed",
            Self::Sox => "South City",

            Self::Fe1 => "Fern Bay Club",
            Self::Fe1r => "Fern Bay Club Reversed",
            Self::Fe2 => "Fern Bay Green",
            Self::Fe2r => "Fern Bay Green Reversed",
            Self::Fe3 => "Fern Bay Gold",
            Self::Fe3r => "Fern Bay Gold Reversed",
            Self::Fe4 => "Fern Bay Black",
            Self::Fe4r => "Fern Bay Black Reversed",
            Self::Fe5 => "Fern Bay RallyX",
            Self::Fe5r => "Fern Bay RallyX Reversed",
            Self::Fe6 => "Fern Bay RallyX Green",
            Self::Fe6r => "Fern Bay RallyX Green Reversed",
            Self::Fex => "Fern Bay",

            Self::Au1 => "AutoX",
            Self::Au2 => "Skidpad",
            Self::Au3 => "Drag Strip",
            Self::Au4 => "8 Lane Drag Strip",

            Self::Ky1 => "Kyoto Oval",
            Self::Ky1r => "Kyoto Oval Reversed",
            Self::Ky2 => "Kyoto National",
            Self::Ky2r => "Kyoto National Reversed",
            Self::Ky3 => "Kyoto Grand Prix Long",
            Self::Ky3r => "Kyoto Grand Prix Long Reversed",
            Self::Kyx => "Kyoto",

            Self::We1 => "Westhill National",
            Self::We1r => "Westhill National Reversed",
            Self::We2 => "Westhill International",
            Self::We2r => "Westhill International Reversed",
            Self::We3 => "Westhill Carpark",
            Self::We4 => "Westhill Karting",
            Self::We4r => "Westhill Karting Reversed",
            Self::We5 => "Westhill Karting National",
            Self::We5r => "Westhill Karting National Reversed",
            Self::Wex => "Westhill",

            Self::As1 => "Aston Cadet",
            Self::As1r => "Aston Cadet Reversed",
            Self::As2 => "Aston Club",
            Self::As2r => "Aston Club Reversed",
            Self::As3 => "Aston National",
            Self::As3r => "Aston National Reversed",
            Self::As4 => "Aston Historic",
            Self::As4r => "Aston Historic Reversed",
            Self::As5 => "Aston Grand Prix",
            Self::As5r => "Aston Grand Prix Reversed",
            Self::As6 => "Aston Grand Touring",
            Self::As6r => "Aston Grand Touring Reversed",
            Self::As7 => "Aston North",
            Self::As7r => "Aston North Reversed",
            Self::Asx => "Aston",

            Self::Ro1 => "Rockingham ISSC",
            Self::Ro2 => "Rockingham National",
            Self::Ro3 => "Rockingham Oval",
            Self::Ro4 => "Rockingham ISSC Long",
            Self::Ro5 => "Rockingham Lake",
            Self::Ro6 => "Rockingham Handling",
            Self::Ro7 => "Rockingham International",
            Self::Ro8 => "Rockingham Historic",
            Self::Ro9 => "Rockingham Historic Short",
            Self::Ro10 => "Rockingham International Long",
            Self::Ro11 => "Rockingham Sportscar",
            Self::Rox => "Rockingham",

            Self::La1 => "Layout Square Long Grid",
            Self::La2 => "Layout Square Wide Grid",

            _ => {
                panic!("Programming error. Unhandled track");
            }
        }
        .to_string()
    }
}
