use crate::identifiers::{ConnectionId, RequestId};

#[derive(Debug, Default, Clone, PartialEq, Eq, insim_core::Decode, insim_core::Encode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "python", pyo3::prelude::pyclass)]
#[repr(u8)]
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

#[derive(Debug, Clone, Default, insim_core::Decode, insim_core::Encode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "python", pyo3::prelude::pyclass)]
/// Connection Leave
pub struct Cnl {
    /// Non-zero if the packet is a packet request or a reply to a request
    pub reqi: RequestId,

    /// Unique connection ID that left
    pub ucid: ConnectionId,

    /// Reason for disconnection
    pub reason: CnlReason,

    /// Number of remaining connections including host
    #[insim(pad_after = 2)]
    pub total: u8,
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
                0, 0,
            ],
            |parsed: Cnl| {
                assert_eq!(parsed.ucid, ConnectionId(4));
                assert_eq!(parsed.total, 14);
                assert!(matches!(parsed.reason, CnlReason::Kicked));
            }
        );
    }
}
