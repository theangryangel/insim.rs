use bytes::{Buf, BufMut};
use insim_core::{
    binrw::{self, binrw},
    FromToBytes,
};

use crate::{identifiers::RequestId, Packet, WithRequestId};

#[binrw]
#[derive(Debug, Default, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[repr(u8)]
#[brw(repr(u8))]
#[non_exhaustive]
/// [Tiny] Subtype
pub enum TinyType {
    /// Keepalive request/response
    #[default]
    None = 0,

    /// Get Version
    Ver = 1,

    /// Close
    Close = 2,

    /// External program requesting a reply (Pong)
    Ping = 3,

    /// Reply to a ping
    Reply = 4,

    /// Vote Cancel
    Vtc = 5,

    /// Send camera position
    Scp = 6,

    /// Send state info
    Sst = 7,

    /// Get time in hundredths (i.e. SMALL_RTP)
    Gth = 8,

    /// Multi-player end
    Mpe = 9,

    /// Get multi-player info
    Ism = 10,

    /// Race end
    Ren = 11,

    /// All players cleared from race
    Clr = 12,

    /// Request NCN for all connections
    Ncn = 13,

    /// Request NPL for all players
    Npl = 14,

    /// Get all results
    Res = 15,

    /// Request a Nlp packet
    Nlp = 16,

    /// Request a Mci packet
    Mci = 17,

    /// Request a Reo packet
    Reo = 18,

    /// Request a Rst packet
    Rst = 19,

    /// Request a Axi packet
    Axi = 20,

    /// Autocross cleared
    Axc = 21,

    /// Request a Rip packet
    Rip = 22,

    /// Request a Nci packet for all guests
    Nci = 23,

    /// Request a Small packet, type = Alc
    Alc = 24,

    /// Request a Axm packet, for the entire layout
    Axm = 25,

    /// Request a Slc packet for all connections
    Slc = 26,

    /// Request a Mal packet
    Mal = 27,

    /// Request a Plh packet
    Plh = 28,

    /// Request a Ipb packet
    Ipb = 29,
}

impl FromToBytes for TinyType {
    fn from_bytes(buf: &mut bytes::Bytes) -> Result<Self, insim_core::Error> {
        let discrim = buf.get_u8();
        let val = match discrim {
            0 => Self::None,
            1 => Self::Ver,
            2 => Self::Close,
            3 => Self::Ping,
            4 => Self::Reply,
            5 => Self::Vtc,
            6 => Self::Scp,
            7 => Self::Sst,
            8 => Self::Gth,
            9 => Self::Mpe,
            10 => Self::Ism,
            11 => Self::Ren,
            12 => Self::Clr,
            13 => Self::Ncn,
            14 => Self::Npl,
            15 => Self::Res,
            16 => Self::Nlp,
            17 => Self::Mci,
            18 => Self::Reo,
            19 => Self::Rst,
            20 => Self::Axi,
            21 => Self::Axc,
            22 => Self::Rip,
            23 => Self::Nci,
            24 => Self::Alc,
            25 => Self::Axm,
            26 => Self::Slc,
            27 => Self::Mal,
            28 => Self::Plh,
            29 => Self::Ipb,
            found => {
                return Err(insim_core::Error::NoVariantMatch {
                    found: found as u64,
                })
            },
        };
        Ok(val)
    }

    fn to_bytes(&self, buf: &mut bytes::BytesMut) -> Result<(), insim_core::Error> {
        let val: u8 = match self {
            Self::None => 0,
            Self::Ver => 1,
            Self::Close => 2,
            Self::Ping => 3,
            Self::Reply => 4,
            Self::Vtc => 5,
            Self::Scp => 6,
            Self::Sst => 7,
            Self::Gth => 8,
            Self::Mpe => 9,
            Self::Ism => 10,
            Self::Ren => 11,
            Self::Clr => 12,
            Self::Ncn => 13,
            Self::Npl => 14,
            Self::Res => 15,
            Self::Nlp => 16,
            Self::Mci => 17,
            Self::Reo => 18,
            Self::Rst => 19,
            Self::Axi => 20,
            Self::Axc => 21,
            Self::Rip => 22,
            Self::Nci => 23,
            Self::Alc => 24,
            Self::Axm => 25,
            Self::Slc => 26,
            Self::Mal => 27,
            Self::Plh => 28,
            Self::Ipb => 29,
        };
        buf.put_u8(val);
        Ok(())
    }
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

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// General purpose Tiny packet
pub struct Tiny {
    /// Non-zero if the packet is a packet request or a reply to a request
    pub reqi: RequestId,

    /// Packet subtype
    pub subt: TinyType,
}

impl Tiny {
    /// Is this a keepalive/ping request?
    pub fn is_keepalive(&self) -> bool {
        self.subt == TinyType::None && self.reqi == RequestId(0)
    }
}

impl_typical_with_request_id!(Tiny);

impl FromToBytes for Tiny {
    fn from_bytes(buf: &mut bytes::Bytes) -> Result<Self, insim_core::Error> {
        let reqi = RequestId::from_bytes(buf)?;
        let subt = TinyType::from_bytes(buf)?;
        Ok(Tiny { reqi, subt })
    }

    fn to_bytes(&self, buf: &mut bytes::BytesMut) -> Result<(), insim_core::Error> {
        self.reqi.to_bytes(buf)?;
        self.subt.to_bytes(buf)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use binrw::{BinRead, BinWrite};

    use super::*;

    #[test]
    fn test_tiny() {
        let parsed = assert_from_to_bytes_bidirectional!(
            Tiny,
            vec![
                0, // reqi
                6  // subt
            ]
        );
        assert_eq!(parsed.subt, TinyType::Scp);
    }
}
