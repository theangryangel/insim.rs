use bytes::{Buf, BufMut};
use insim_core::{
    binrw::{self, binrw},
    Error, ReadWriteBuf,
};

use crate::identifiers::{ConnectionId, RequestId};

#[binrw]
#[derive(Debug, Default, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[repr(u8)]
#[brw(repr(u8))]
#[non_exhaustive]
/// Used within [Cnl] to indicate the leave reason.
pub enum CnlReason {
    #[default]
    /// None
    Disco = 0,

    /// Timeout
    Timeout = 1,

    /// Lost Connection
    LostConn = 2,

    /// Kicked
    Kicked = 3,

    /// Banned
    Banned = 4,

    /// Security
    Security = 5,

    /// Cheat Protection
    Cpw = 6,

    /// Out of sync with host
    Oos = 7,

    /// Join out of sync - initial sync failed
    Joos = 8,

    /// Invalid packet
    Hack = 9,
}

impl ReadWriteBuf for CnlReason {
    fn read_buf(buf: &mut bytes::Bytes) -> Result<Self, insim_core::Error> {
        let val = match u8::read_buf(buf)? {
            0 => Self::Disco,
            1 => Self::Timeout,
            2 => Self::LostConn,
            3 => Self::Kicked,
            4 => Self::Banned,
            5 => Self::Security,
            6 => Self::Cpw,
            7 => Self::Oos,
            8 => Self::Joos,
            9 => Self::Hack,
            found => {
                return Err(Error::NoVariantMatch {
                    found: found as u64,
                })
            },
        };
        Ok(val)
    }

    fn write_buf(&self, buf: &mut bytes::BytesMut) -> Result<(), insim_core::Error> {
        let discrim: u8 = match self {
            Self::Disco => 0,
            Self::Timeout => 1,
            Self::LostConn => 2,
            Self::Kicked => 3,
            Self::Banned => 4,
            Self::Security => 5,
            Self::Cpw => 6,
            Self::Oos => 7,
            Self::Joos => 8,
            Self::Hack => 9,
        };
        discrim.write_buf(buf)?;
        Ok(())
    }
}

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Connection Leave
pub struct Cnl {
    /// Non-zero if the packet is a packet request or a reply to a request
    pub reqi: RequestId,

    /// Unique connection ID that left
    pub ucid: ConnectionId,

    /// Reason for disconnection
    pub reason: CnlReason,

    /// Number of remaining connections including host
    #[brw(pad_after = 2)]
    pub total: u8,
}

impl ReadWriteBuf for Cnl {
    fn read_buf(buf: &mut bytes::Bytes) -> Result<Self, Error> {
        let reqi = RequestId::read_buf(buf)?;
        let ucid = ConnectionId::read_buf(buf)?;
        let reason = CnlReason::read_buf(buf)?;
        let total = u8::read_buf(buf)?;
        buf.advance(2);
        Ok(Self {
            reqi,
            ucid,
            reason,
            total,
        })
    }

    fn write_buf(&self, buf: &mut bytes::BytesMut) -> Result<(), Error> {
        self.reqi.write_buf(buf)?;
        self.ucid.write_buf(buf)?;
        self.reason.write_buf(buf)?;
        self.total.write_buf(buf)?;
        buf.put_bytes(0, 2);
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_cnl() {
        assert_from_to_bytes!(
            Cnl,
            [
                0,  // reqi
                4,  // ucid
                3,  // reason
                14, // total
                0,  // sp2
                0,  // sp3
            ],
            |parsed: Cnl| {
                assert_eq!(parsed.ucid, ConnectionId(4));
                assert_eq!(parsed.total, 14);
                assert!(matches!(parsed.reason, CnlReason::Kicked));
            }
        );
    }
}
