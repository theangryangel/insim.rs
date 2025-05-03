use crate::{identifiers::RequestId, WithRequestId};

#[derive(Debug, Default, Clone, insim_core::Decode, insim_core::Encode)]
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

#[derive(Debug, Clone, Default, insim_core::Decode, insim_core::Encode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Send Screenshot - instructional and informational.
pub struct Ssh {
    /// Non-zero if the packet is a packet request or a reply to a request
    pub reqi: RequestId,

    /// Result code
    #[read_write_buf(pad_after = 4)]
    pub error: SshError,

    /// Screenshot name.
    /// Not really ascii, but given we dont have control over the naming convention we can
    /// probably just abuse the fact that LFS only generates ASCII compatible file names.
    #[read_write_buf(ascii(length = 32, trailing_nul = true))]
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

#[cfg(test)]
mod test {
    use bytes::{BufMut, BytesMut};

    use super::*;

    #[test]
    fn test_ssh() {
        let mut data = BytesMut::new();
        data.extend_from_slice(&[
            2, // reqi
            0, // error
            0, 0, 0, 0,
        ]);
        data.extend_from_slice(b"lfs_00000001");
        data.put_bytes(0, 20);
        assert_from_to_bytes!(Ssh, data.as_ref(), |ssh: Ssh| {
            assert!(matches!(ssh.error, SshError::Ok));
            assert_eq!(ssh.name, "lfs_00000001");
        });
    }
}
