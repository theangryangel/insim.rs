use insim_core::{identifiers::RequestId, prelude::*};

#[cfg(feature = "serde")]
use serde::Serialize;

/// Send a HostSelect to the relay in order to start receiving information about the selected host.
#[derive(Debug, Clone, Default, InsimEncode, InsimDecode)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct HostSelect {
    #[insim(pad_bytes_after = "1")]
    pub reqi: RequestId,

    #[insim(bytes = "32")]
    pub hname: String,

    #[insim(bytes = "16")]
    pub admin: String,

    #[insim(bytes = "16")]
    pub spec: String,
}
