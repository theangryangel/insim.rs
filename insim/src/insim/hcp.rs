use insim_core::binrw::{self, binrw};

use crate::identifiers::RequestId;

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[bw(assert(*h_mass <= 200, "h_mass must be <= 200"))]
#[bw(assert(*h_tres <= 50, "h_tres must be <= 50"))]
/// Used within [Hcp] to apply handicaps to a vehicle.
pub struct HcpCarHandicap {
    /// 0 to 200 - added mass (kg)
    pub h_mass: u8,

    /// 0 to  50 - intake restriction
    pub h_tres: u8,
}

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Vehicle Handicaps
/// You can send a packet to add mass and restrict the intake on each car model
/// The same restriction applies to all drivers using a particular car model
/// This can be useful for creating multi class hosts.
/// The info field is indexed by the vehicle. i.e. XF GTI = 0, XR GT = 1, etc.
pub struct Hcp {
    #[brw(pad_after = 1)]
    /// Non-zero if the packet is a packet request or a reply to a request
    pub reqi: RequestId,

    /// H_Mass and H_TRes for each car: : XF GTI = 0 / XR GT = 1 etc
    // TODO make this work with [Vehicle]
    pub info: [HcpCarHandicap; 32],
}
