use insim_core::{
    binrw::{self, binrw},
    string::{binrw_parse_codepage_string_until_eof, binrw_write_codepage_string},
    FromToCodepageBytes,
};

use crate::identifiers::{ConnectionId, RequestId};

/// Enum for the result field of [Acr].
#[binrw]
#[brw(repr(u8))]
#[repr(u8)]
#[derive(Debug, Default, Clone, insim_macros::ReadWriteBuf)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
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
#[binrw]
#[derive(Debug, Clone, Default, insim_macros::ReadWriteBuf)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Acr {
    /// Non-zero if the packet is a packet request or a reply to a request
    #[brw(pad_after = 1)]
    #[read_write_buf(pad_after = 1)]
    pub reqi: RequestId,

    /// Unique connection identifier
    pub ucid: ConnectionId,

    /// Is the user an admin?
    #[br(map = |x: u8| x != 0)]
    #[bw(map = |&x| x as u8)]
    pub admin: bool,

    /// Result
    #[brw(pad_after = 1)]
    #[read_write_buf(pad_after = 1)]
    pub result: AcrResult,

    /// Command
    #[bw(write_with = binrw_write_codepage_string::<64, _>, args(false, 4))]
    #[br(parse_with = binrw_parse_codepage_string_until_eof)]
    #[read_write_buf(
        read_with = "|buf| { String::from_codepage_bytes(buf, 64) }",
        write_with = "|msg: &String, buf| { msg.to_codepage_bytes_aligned(buf, 64, 4) }"
    )]
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
                0,  // ReqI
                0,  // Zero
                2,  // UCID
                1,  // Admin
                1,  // Result
                0,  // Sp3
                47, // Text[64]
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
