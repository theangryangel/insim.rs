use insim_core::{
    binrw::{self, binrw},
    ReadWriteBuf,
};

use crate::identifiers::RequestId;

/// Response to a [super::admin_request::Arq] packet, indicating if we are logged in as an administrative user on
/// the selected host.
#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Arp {
    /// Optional request identifier. If a request identifier was sent in the request, it will be
    /// included in any relevant response packet.
    pub reqi: RequestId,

    /// true if we are an admin
    #[br(map = |x: u8| x != 0)]
    #[bw(map = |&x| x as u8)]
    pub admin: bool,
}

impl ReadWriteBuf for Arp {
    fn read_buf(buf: &mut bytes::Bytes) -> Result<Self, insim_core::Error> {
        let reqi = RequestId::read_buf(buf)?;
        let admin = u8::read_buf(buf)? != 0;

        Ok(Self { reqi, admin })
    }

    fn write_buf(&self, buf: &mut bytes::BytesMut) -> Result<(), insim_core::Error> {
        self.reqi.write_buf(buf)?;
        (self.admin as u8).write_buf(buf)?;
        Ok(())
    }
}
