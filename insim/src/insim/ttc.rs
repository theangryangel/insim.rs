use crate::{
    Packet, WithRequestId,
    identifiers::{ConnectionId, RequestId},
};

#[derive(Debug, Default, Clone, insim_core::Decode, insim_core::Encode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
#[non_exhaustive]
/// Subtype for the [Ttc] packet.
pub enum TtcType {
    /// Request the current layout editor selection ([`Axm`](crate::insim::Axm)).
    #[default]
    Sel = 1,

    /// Start streaming layout selection changes.
    SelStart = 2,

    /// Stop streaming layout selection changes.
    SelStop = 3,
}

#[derive(Debug, Clone, Default, insim_core::Decode, insim_core::Encode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Target-to-connection packet for selection-related requests.
///
/// - Routes a request to a specific connection.
/// - The meaning of `b1`..`b3` depends on the subtype.
pub struct Ttc {
    /// Request identifier echoed by replies.
    pub reqi: RequestId,

    /// Subtype describing the request.
    pub subt: TtcType,

    /// Connection unique id to target (0 = local).
    pub ucid: ConnectionId,

    /// Extra data byte (meaning depends on `subt`).
    pub b1: u8,

    /// Extra data byte (meaning depends on `subt`).
    pub b2: u8,

    /// Extra data byte (meaning depends on `subt`).
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
