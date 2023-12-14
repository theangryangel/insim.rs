use insim_core::{
    binrw::{self, binrw},
    identifiers::RequestId,
};

#[cfg(feature = "serde")]
use serde::Serialize;

/// Ask the relay if we are logged in as an administrative user on the selected host. A
/// [super::admin_response::AdminResponse] is sent back by the relay.
#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct AdminRequest {
    #[brw(pad_after = 1)]
    pub reqi: RequestId,
}
