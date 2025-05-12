use crate::identifiers::RequestId;

/// Send a Sel to the relay in order to start receiving information about the selected host.
#[derive(Debug, Clone, Default, insim_core::Decode, insim_core::Encode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Sel {
    /// If Non-zero LFS World relay will reply with a [crate::Packet::Ver]
    #[insim(pad_after = 1)]
    pub reqi: RequestId,

    /// Name of host to select
    #[insim(codepage(length = 32))]
    pub hname: String,

    /// Administrative password.
    #[insim(codepage(length = 16))]
    pub admin: String,

    /// Spectator password.
    #[insim(codepage(length = 16))]
    pub spec: String,
}
