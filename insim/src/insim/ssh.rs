use crate::{identifiers::RequestId, WithRequestId};

#[derive(Debug, Default, Clone, insim_macros::ReadWriteBuf)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[repr(u8)]
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

#[derive(Debug, Clone, Default, insim_macros::ReadWriteBuf)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Send Screenshot - instructional and informational.
pub struct Ssh {
    /// Non-zero if the packet is a packet request or a reply to a request
    pub reqi: RequestId,

    /// Result code
    #[read_write_buf(pad_after = 4)]
    pub error: SshError,

    /// Screenshot file path.
    // FIXME: Probably not really ascii. definitely not a codepage. Probably wchar_t?
    #[read_write_buf(ascii(length = 32))]
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
