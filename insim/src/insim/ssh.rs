use insim_core::{
    binrw::{self, binrw},
    string::{binrw_parse_codepage_string, binrw_write_codepage_string},
};

use crate::{identifiers::RequestId, WithRequestId};

#[binrw]
#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[repr(u8)]
#[brw(repr(u8))]
#[non_exhaustive]
/// Errors occurred during a [Ssh] request.
pub enum SshError {
    #[default]
    /// No error
    Ok = 0,

    /// This is a dedicated server. Screenshot unavailable.
    Dedicated = 1,

    /// Screenshot corrupted.
    Corrupted = 2,

    /// Could not save.
    NoSave = 3,
}

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Send Screenshot - instructional and informational.
pub struct Ssh {
    /// Non-zero if the packet is a packet request or a reply to a request
    pub reqi: RequestId,

    /// Result code
    #[brw(pad_after = 4)]
    pub error: SshError,

    /// Screenshot file path.
    #[br(parse_with = binrw_parse_codepage_string::<32, _>)]
    #[bw(write_with = binrw_write_codepage_string::<32, _>)]
    pub name: String,
}

impl_typical_with_request_id!(Ssh);

impl From<SshError> for Ssh {
    fn from(error: SshError) -> Self {
        Self {
            error,
            ..Default::default()
        }
    }
}

impl WithRequestId for SshError {
    fn with_request_id<R: Into<crate::identifiers::RequestId>>(
        self,
        reqi: R,
    ) -> impl Into<crate::Packet> + std::fmt::Debug {
        Ssh {
            reqi: reqi.into(),
            error: self,
            ..Default::default()
        }
    }
}
