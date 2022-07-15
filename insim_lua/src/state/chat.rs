use chrono::prelude::*;

#[derive(Debug)]
pub struct Chat {
    pub at: DateTime<Utc>,
    pub ucid: u8,
    pub body: String,
}

impl Chat {
    pub fn new(ucid: u8, body: String) -> Self {
        Self {
            at: Utc::now(),
            ucid,
            body,
        }
    }
}
