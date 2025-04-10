use bytes::{Buf, BufMut};
use insim_core::{
    binrw::{self, binrw},
    FromToBytes,
};

use super::CameraView;
use crate::identifiers::{PlayerId, RequestId};

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Set Car Camera
pub struct Scc {
    /// Non-zero if the packet is a packet request or a reply to a request
    #[brw(pad_after = 1)]
    pub reqi: RequestId,

    /// Player ID
    pub viewplid: PlayerId,

    /// How to manipulate the camera. See [CameraView].
    #[brw(pad_after = 2)]
    pub ingamecam: CameraView,
}

impl FromToBytes for Scc {
    fn from_bytes(buf: &mut bytes::Bytes) -> Result<Self, insim_core::Error> {
        let reqi = RequestId::from_bytes(buf)?;
        buf.advance(1);
        let viewplid = PlayerId::from_bytes(buf)?;
        let ingamecam = CameraView::from_bytes(buf)?;
        buf.advance(2);
        Ok(Self {
            reqi,
            viewplid,
            ingamecam,
        })
    }

    fn to_bytes(&self, buf: &mut bytes::BytesMut) -> Result<(), insim_core::Error> {
        self.reqi.to_bytes(buf)?;
        buf.put_bytes(0, 1);
        self.viewplid.to_bytes(buf)?;
        self.ingamecam.to_bytes(buf)?;
        buf.put_bytes(0, 2);
        Ok(())
    }
}

impl_typical_with_request_id!(Scc);

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_scc() {
        assert_from_to_bytes!(
            Scc,
            [
                1, // reqi
                0, 1, // viewplid
                3, // ingamecam
                0, 0,
            ],
            |parsed: Scc| {
                assert_eq!(parsed.reqi, RequestId(1));
            }
        );
    }
}
