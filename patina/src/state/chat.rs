use chrono::prelude::*;
use insim::protocol::identifiers::ConnectionId;
use serde::Serialize;

#[derive(Debug, Default, Clone, Serialize)]
pub struct Chat {
    pub at: DateTime<Utc>,
    pub ucid: ConnectionId,
    pub pname: String,
    pub uname: String,
    pub colour: String,
    pub body: String,
}

impl Chat {
    pub fn new(ucid: ConnectionId, body: String) -> Self {
        Self {
            at: Utc::now(),
            ucid,
            body,
            ..Default::default()
        }
    }
}
