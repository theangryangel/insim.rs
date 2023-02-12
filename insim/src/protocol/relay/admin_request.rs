use insim_core::{identifiers::RequestId, prelude::*};

#[cfg(feature = "serde")]
use serde::Serialize;

/// Ask the relay if we are logged in as an administrative user on the selected host. A
/// [AdminResponse] is sent back by the relay.
#[derive(Debug, Clone, Default, InsimEncode, InsimDecode)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct AdminRequest {
    #[insim(pad_bytes_after = "1")]
    pub reqi: RequestId,
}
