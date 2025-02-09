use bytes::Buf;
use insim_core::{binrw::{self, binrw}, FromToBytes};

use crate::identifiers::RequestId;

/// Ask the relay if we are logged in as an administrative user on the selected host. A
/// [super::admin_response::Arp] is sent back by the relay.
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Arq {
    /// Non-zero if the packet is a packet request or a reply to a request
    pub reqi: RequestId,
}

impl FromToBytes for Arq {
    fn from_bytes(buf: &mut bytes::Bytes) -> Result<Self, insim_core::Error> {
        let reqi = RequestId::from_bytes(buf)?;
        buf.advance(1);
        Ok(Self {
            reqi 
        })
    }

    fn to_bytes(&self, buf: &mut bytes::BytesMut) -> Result<usize, insim_core::Error> {
        self.reqi.to_bytes(buf)?;
        buf.put_bytes(0, 1);
        Ok(2)
    }
}
