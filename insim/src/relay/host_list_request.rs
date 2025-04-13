use bytes::{Buf, BufMut};
use insim_core::{
    binrw::{self, binrw},
    ReadWriteBuf,
};

use crate::identifiers::RequestId;

/// Request a list of available hosts from the Insim Relay. After sending this packet the relay
/// will respond with a [super::host_list::Hos] packet.
#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Hlr {
    /// Non-zero if the packet is a packet request or a reply to a request
    pub reqi: RequestId,
}

impl ReadWriteBuf for Hlr {
    fn read_buf(buf: &mut bytes::Bytes) -> Result<Self, insim_core::Error> {
        let reqi = RequestId::read_buf(buf)?;
        buf.advance(1);
        Ok(Self { reqi })
    }

    fn write_buf(&self, buf: &mut bytes::BytesMut) -> Result<(), insim_core::Error> {
        self.reqi.write_buf(buf)?;
        buf.put_bytes(0, 1);
        Ok(())
    }
}
