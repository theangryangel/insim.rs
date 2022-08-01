use chrono::prelude::*;
use insim::protocol::identifiers::ConnectionId;

#[derive(Debug)]
pub struct Chat {
    pub at: DateTime<Utc>,
    pub ucid: ConnectionId,
    pub body: String,
}

impl Chat {
    pub fn new(ucid: ConnectionId, body: String) -> Self {
        Self {
            at: Utc::now(),
            ucid,
            body,
        }
    }
}
