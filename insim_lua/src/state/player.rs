use insim::protocol::position::FixedPoint;

#[derive(Debug, Default)]
pub struct Player {
    pub pname: String,
    pub plate: String,

    pub xyz: FixedPoint,

    pub in_pitlane: bool,
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
