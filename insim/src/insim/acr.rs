use crate::identifiers::{ConnectionId, RequestId};

/// Result of an admin command.
#[repr(u8)]
#[derive(Debug, Default, Clone, insim_core::Decode, insim_core::Encode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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

/// Admin command report.
///
/// - Contains the raw command text and result.
#[derive(Debug, Clone, Default, insim_core::Decode, insim_core::Encode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Acr {
    /// Request identifier echoed by replies.
    #[insim(pad_after = 1)]
    pub reqi: RequestId,

    /// Connection that issued the command.
    pub ucid: ConnectionId,

    /// Whether the user is an admin.
    pub admin: bool,

    /// Command result.
    #[insim(pad_after = 1)]
    pub result: AcrResult,

    /// Command text.
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
