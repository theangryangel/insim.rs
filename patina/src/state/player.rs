use insim::protocol::position::Point;
use serde::Serialize;

#[derive(Debug, Clone, Default, Serialize)]
pub struct Player {
    pub pname: String,
    pub plate: String,

    pub xyz: Point<i32>,

    pub in_pitlane: bool,

    pub lap: u16,

    pub position: u8,

    pub node: u16,

    pub speed: u16,

    pub colour: String,
}

impl From<&insim::protocol::insim::Npl> for Player {
    fn from(data: &insim::protocol::insim::Npl) -> Self {
        Self {
            pname: data.pname.to_lossy_string(),
            plate: data.plate.to_lossy_string(),
            ..Default::default()
        }
    }
}
