use insim_core::{
    binrw::{self, binrw},
    FromToBytes,
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

impl FromToBytes for Arp {
    fn from_bytes(buf: &mut bytes::Bytes) -> Result<Self, insim_core::Error> {
        let reqi = RequestId::from_bytes(buf)?;
        let admin = u8::from_bytes(buf)? != 0;

        Ok(Self { reqi, admin })
    }

    fn to_bytes(&self, buf: &mut bytes::BytesMut) -> Result<(), insim_core::Error> {
        self.reqi.to_bytes(buf)?;
        (self.admin as u8).to_bytes(buf)?;
        Ok(())
    }
}
