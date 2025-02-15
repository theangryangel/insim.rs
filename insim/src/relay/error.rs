use insim_core::{
    binrw::{self, binrw},
    FromToBytes,
};

use crate::identifiers::RequestId;

/// Enum of possible errors  that the Insim Relay can respond with.
#[binrw]
#[brw(repr(u8))]
#[derive(Debug, Clone, Default, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[repr(u8)]
pub enum RelayErrorKind {
    #[default]
    /// None
    None = 0,

    /// Packet length or structure is invalid.
    InvalidPacketLength = 1,

    /// Packet type cannot be forward to the host.
    InvalidPacketType = 2,

    /// Invalid hostname
    InvalidHostname = 3,

    /// Administrative password was rejected.
    BadAdminPassword = 4,

    /// Spectator password was rejected.
    BadSpectatorPassword = 5,

    /// Spectator password was required but not provided.
    MissingSpectatorPassword = 6,
}

impl FromToBytes for RelayErrorKind {
    fn from_bytes(buf: &mut bytes::Bytes) -> Result<Self, insim_core::Error> {
        let discrim = u8::from_bytes(buf)?;
        let kind = match discrim {
            0 => Self::None,
            1 => Self::InvalidPacketLength,
            2 => Self::InvalidPacketType,
            3 => Self::InvalidHostname,
            4 => Self::BadAdminPassword,
            5 => Self::BadSpectatorPassword,
            6 => Self::MissingSpectatorPassword,
            found => {
                return Err(insim_core::Error::NoVariantMatch {
                    found: found.into(),
                })
            },
        };

        Ok(kind)
    }

    fn to_bytes(&self, buf: &mut bytes::BytesMut) -> Result<(), insim_core::Error> {
        (*self as u8).to_bytes(buf)?;
        Ok(())
    }
}

/// The relay will send this packet when it encounters an error.
#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Error {
    /// Non-zero if the packet is a packet request or a reply to a request
    pub reqi: RequestId,

    /// The error
    pub err: RelayErrorKind,
}

impl FromToBytes for Error {
    fn from_bytes(buf: &mut bytes::Bytes) -> Result<Self, insim_core::Error> {
        let reqi = RequestId::from_bytes(buf)?;
        let err = RelayErrorKind::from_bytes(buf)?;
        Ok(Self { reqi, err })
    }

    fn to_bytes(&self, buf: &mut bytes::BytesMut) -> Result<(), insim_core::Error> {
        self.reqi.to_bytes(buf)?;
        self.err.to_bytes(buf)?;
        Ok(())
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Encountered a Relay Error: {:?}", self.err)
    }
}

impl std::error::Error for Error {}
