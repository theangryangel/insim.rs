use crate::{
    identifiers::{ConnectionId, RequestId},
    Packet, WithRequestId,
};

#[derive(Debug, Default, Clone, insim_macros::ReadWriteBuf)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[repr(u8)]
#[non_exhaustive]
/// [Ttc] subtype.
pub enum TtcType {
    /// Send Axm for the current layout editor selection
    #[default]
    Sel = 1,

    /// Send Axm every time the selection changes
    SelStart = 2,

    /// Stop sending Axm's
    SelStop = 3,
}

#[derive(Debug, Clone, Default, insim_macros::ReadWriteBuf)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// General purpose Target To Connection packet
/// b1..b3 may be used in various ways, depending on the subtype
pub struct Ttc {
    /// Non-zero if the packet is a packet request or a reply to a request
    pub reqi: RequestId,

    /// Subtype
    pub subt: TtcType,

    /// Connection unique ID to target
    pub ucid: ConnectionId,

    // TODO: Fix this. It should be rolled into TtcType
    /// B1, B2, B3 may be used in various ways depending on SubT
    pub b1: u8,

    /// B1, B2, B3 may be used in various ways depending on SubT
    pub b2: u8,

    /// B1, B2, B3 may be used in various ways depending on SubT
    pub b3: u8,
}

impl From<TtcType> for Packet {
    fn from(value: TtcType) -> Self {
        Self::Ttc(Ttc {
            subt: value,
            ..Default::default()
        })
    }
}

impl_typical_with_request_id!(Ttc);

impl WithRequestId for TtcType {
    fn with_request_id<R: Into<RequestId>>(
        self,
        reqi: R,
    ) -> impl Into<crate::Packet> + std::fmt::Debug {
        Ttc {
            reqi: reqi.into(),
            subt: self,
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_ttc() {
        assert_from_to_bytes!(
            Ttc,
            [
                7, // reqi
                2, // subt
                5, // ucid
                1, // b1
                2, // b2
                3, // b3
            ],
            |ttc: Ttc| {
                assert_eq!(ttc.reqi, RequestId(7));
                assert_eq!(ttc.ucid, ConnectionId(5));
                assert!(matches!(ttc.subt, TtcType::SelStart));
            }
        );
    }
}
