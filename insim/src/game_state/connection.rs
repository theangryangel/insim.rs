use crate::core::{identifiers::ConnectionId, identifiers::PlayerId, point::Point};
use crate::packets::insim::Ncn;

use multi_index::MultiIndex;

#[derive(MultiIndex, Default, Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Connection {
    #[multi_index(how = "ordered", unique)]
    pub connection_id: ConnectionId,

    #[multi_index(how = "ordered", unique, ignore_none)]
    pub player_id: Option<PlayerId>,

    /// Connection username
    pub uname: String,

    /// Connection has admin rights
    pub admin: bool,

    /// Connection flags
    pub connection_flags: u8,

    /// player name
    pub pname: String,

    /// player plate
    pub plate: Option<String>,

    pub xyz: Option<Point<i32>>,

    pub in_pitlane: bool,

    pub lap: Option<u16>,

    pub position: Option<u8>,

    pub node: u16,

    pub speed: u16,
}

impl From<&Ncn> for Connection {
    fn from(data: &Ncn) -> Self {
        Self {
            uname: data.uname.clone(),
            admin: data.admin,
            connection_flags: data.flags,
            connection_id: data.ucid,
            player_id: None,
            pname: data.pname.to_string(),
            ..Default::default()
        }
    }
}
