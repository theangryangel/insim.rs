use chrono::prelude::*;
use insim::core::identifiers::ConnectionId;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Chat {
    pub at: DateTime<Utc>,
    pub ucid: ConnectionId,
    pub pname: String,
    pub uname: String,
    pub colour: String,
    pub body: String,
}

impl Default for Chat {
    fn default() -> Self {
        Self {
            at: Utc::now(),
            ucid: ConnectionId::default(),
            pname: "".into(),
            uname: "".into(),
            colour: "".into(),
            body: "".into(),
        }
    }
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
