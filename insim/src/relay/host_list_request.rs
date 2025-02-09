use bytes::{Buf, BufMut};
use insim_core::{binrw::{self, binrw}, FromToBytes};

use crate::identifiers::RequestId;

/// Request a list of available hosts from the Insim Relay. After sending this packet the relay
/// will respond with a [super::host_list::Hos] packet.
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Hlr {
    /// Non-zero if the packet is a packet request or a reply to a request
    pub reqi: RequestId,
}

impl FromToBytes for Hlr {
    fn from_bytes(buf: &mut bytes::Bytes) -> Result<Self, insim_core::Error> {
        let reqi = RequestId::from_bytes(buf)?;
        buf.advance(1);
        Ok(Self{ reqi })
    }

    fn to_bytes(&self, buf: &mut bytes::BytesMut) -> Result<usize, insim_core::Error> {
        self.reqi.to_bytes(buf)?;
        buf.put_bytes(0, 1);
        Ok(())
    }
}
