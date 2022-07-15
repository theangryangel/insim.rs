use insim::protocol::position::FixedPoint;

#[derive(Debug)]
pub(crate) struct Player {
    pub pname: String,
    pub plate: String,

    pub xyz: FixedPoint,
}

impl From<&insim::protocol::insim::Npl> for Player {
    fn from(data: &insim::protocol::insim::Npl) -> Self {
        Self {
            pname: data.pname.to_lossy_string(),
            plate: data.plate.to_lossy_string(),
            xyz: FixedPoint::default(),
        }
    }
}
