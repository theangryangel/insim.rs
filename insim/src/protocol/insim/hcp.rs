use insim_core::prelude::*;

#[cfg(feature = "serde")]
use serde::Serialize;

use crate::protocol::identifiers::RequestId;

#[derive(Debug, InsimEncode, InsimDecode, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Used within [Hcp] to apply handicaps to a vehicle.
pub struct HcpCarHandicap {
    pub added_mass: u8,
    pub intake_restriction: u8,
}

#[derive(Debug, InsimEncode, InsimDecode, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Vehicle Handicaps
/// You can send a packet to add mass and restrict the intake on each car model
/// The same restriction applies to all drivers using a particular car model
/// This can be useful for creating multi class hosts.
/// The info field is indexed by the vehicle. i.e. XF GTI = 0, XR GT = 1, etc.
pub struct Hcp {
    #[deku(pad_bytes_after = "1")]
    pub reqi: RequestId,

    #[deku(count = "32")]
    pub info: Vec<HcpCarHandicap>,
}
