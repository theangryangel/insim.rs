use insim::protocol::position::FixedPoint;

#[derive(Debug)]
pub struct Player {
    pub pname: String,
    pub plate: String,

    pub xyz: FixedPoint,

    pub in_pitlane: bool,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            pname: String::new(),
            plate: String::new(),
            xyz: FixedPoint::default(),
            in_pitlane: false,
        }
    }
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
