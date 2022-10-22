#[derive(Debug, Clone)]
pub struct Connection {
    pub uname: String,
    pub admin: bool,
    pub flags: u8,
}

impl From<&insim::protocol::insim::Ncn> for Connection {
    fn from(data: &insim::protocol::insim::Ncn) -> Self {
        Self {
            uname: data.uname.clone(),
            admin: data.admin > 0,
            flags: data.flags,
        }
    }
}
