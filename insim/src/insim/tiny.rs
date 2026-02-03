use crate::{Packet, WithRequestId, identifiers::RequestId};

#[derive(Debug, Default, Clone, Eq, PartialEq, insim_core::Decode, insim_core::Encode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[repr(u8)]
#[non_exhaustive]
/// Subtype for the [Tiny] packet.
pub enum TinyType {
    /// Keepalive request/response.
    #[default]
    None = 0,

    /// Request version information.
    Ver = 1,

    /// Close the InSim connection.
    Close = 2,

    /// Request a ping reply.
    Ping = 3,

    /// Reply to a ping request.
    Reply = 4,

    /// Vote cancelled or request vote cancellation.
    Vtc = 5,

    /// Request camera position.
    Scp = 6,

    /// Request state information.
    Sst = 7,

    /// Request race time (reply is [`SmallType::Rtp`](crate::insim::SmallType::Rtp)).
    Gtm = 8,

    /// Multiplayer ended.
    Mpe = 9,

    /// Request multiplayer info (reply is [`Ism`](crate::insim::Ism)).
    Ism = 10,

    /// Race end.
    Ren = 11,

    /// All players cleared from race.
    Clr = 12,

    /// Request all connection info ([`Ncn`](crate::insim::Ncn)).
    Ncn = 13,

    /// Request all players ([`Npl`](crate::insim::Npl)).
    Npl = 14,

    /// Request all results ([`Res`](crate::insim::Res)).
    Res = 15,

    /// Request node and lap info ([`Nlp`](crate::insim::Nlp)).
    Nlp = 16,

    /// Request multi-car info ([`Mci`](crate::insim::Mci)).
    Mci = 17,

    /// Request reorder information ([`Reo`](crate::insim::Reo)).
    Reo = 18,

    /// Request race start info ([`Rst`](crate::insim::Rst)).
    Rst = 19,

    /// Request autocross layout info ([`Axi`](crate::insim::Axi)).
    Axi = 20,

    /// Autocross cleared.
    Axc = 21,

    /// Request replay info ([`Rip`](crate::insim::Rip)).
    Rip = 22,

    /// Request all connection info (host only) ([`Nci`](crate::insim::Nci)).
    Nci = 23,

    /// Request allowed cars ([`SmallType::Alc`](crate::insim::SmallType::Alc)).
    Alc = 24,

    /// Request the full layout ([`Axm`](crate::insim::Axm)).
    Axm = 25,

    /// Request all selected cars ([`Slc`](crate::insim::Slc)).
    Slc = 26,

    /// Request allowed mods ([`Mal`](crate::insim::Mal)).
    Mal = 27,

    /// Request player handicaps ([`Plh`](crate::insim::Plh)).
    Plh = 28,

    /// Request IP bans ([`Ipb`](crate::insim::Ipb)).
    Ipb = 29,

    /// Request local car lights ([`SmallType::Lcl`](crate::insim::SmallType::Lcl)).
    Lcl = 30,
}

impl From<TinyType> for Packet {
    fn from(value: TinyType) -> Self {
        Self::Tiny(Tiny {
            subt: value,
            ..Default::default()
        })
    }
}

impl WithRequestId for TinyType {
    fn with_request_id<R: Into<RequestId>>(self, reqi: R) -> impl Into<Packet> + std::fmt::Debug {
        Tiny {
            reqi: reqi.into(),
            subt: self,
        }
    }
}

#[derive(Debug, Clone, Default, insim_core::Decode, insim_core::Encode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// General purpose request/reply packet.
///
/// - Used for lightweight requests and notifications.
/// - The meaning is defined by the `subt` value.
pub struct Tiny {
    /// Request identifier echoed by replies.
    pub reqi: RequestId,

    /// Subtype describing the request or notification.
    pub subt: TinyType,
}

impl Tiny {
    /// Is this a keepalive/ping request?
    pub fn is_keepalive(&self) -> bool {
        self.subt == TinyType::None && self.reqi == RequestId(0)
    }
}

impl_typical_with_request_id!(Tiny);

#[cfg(test)]
mod tests {
    use bytes::{BufMut, BytesMut};
    use insim_core::Decode;

    use super::*;

    #[test]
    fn test_tiny_type_mal() {
        let mut buf = BytesMut::new();
        buf.put_u8(27);

        let ty = TinyType::decode(&mut buf.freeze()).unwrap();
        assert!(matches!(ty, TinyType::Mal));
    }

    #[test]
    fn test_tiny() {
        assert_from_to_bytes!(
            Tiny,
            vec![
                0, // reqi
                6  // subt
            ],
            |parsed: Tiny| {
                assert_eq!(parsed.subt, TinyType::Scp);
            }
        );
    }
}
