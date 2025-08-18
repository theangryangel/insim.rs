use crate::identifiers::{ConnectionId, RequestId};

/// Enum for the result field of [Acr].
#[repr(u8)]
#[derive(Debug, Default, Clone, insim_core::Decode, insim_core::Encode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "python", pyo3::prelude::pyclass)]
#[non_exhaustive]
pub enum AcrResult {
    /// Command was processed
    #[default]
    Processed = 1,

    /// Command was rejected
    Rejected = 2,

    /// Unknown command
    UnknownCommand = 3,
}

/// Admin Command Report: A user typed an admin command - variable size
#[derive(Debug, Clone, Default, insim_core::Decode, insim_core::Encode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "python", pyo3::prelude::pyclass)]
pub struct Acr {
    /// Non-zero if the packet is a packet request or a reply to a request
    #[insim(pad_after = 1)]
    pub reqi: RequestId,

    /// Unique connection identifier
    pub ucid: ConnectionId,

    /// Is the user an admin?
    pub admin: bool,

    /// Result
    #[insim(pad_after = 1)]
    pub result: AcrResult,

    /// Command
    #[insim(codepage(length = 64, align_to = 4, trailing_nul = true))]
    pub text: String,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_acr() {
        assert_from_to_bytes!(
            Acr,
            [
                0, // reqi
                0, 2, // ucid
                1, // admin
                1, // result
                0, 47, // text[64]
                108, 97, 112, 115, 32, 50, 0,
            ],
            |acr: Acr| {
                assert_eq!(acr.reqi, RequestId(0));
                assert_eq!(acr.ucid, ConnectionId(2));
                assert!(acr.admin);
                assert!(matches!(acr.result, AcrResult::Processed));
                assert_eq!(acr.text, "/laps 2");
            }
        );
    }
}
