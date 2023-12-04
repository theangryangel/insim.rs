use insim_core::{
    binrw::{self, binrw},
    identifiers::RequestId,
};

#[cfg(feature = "serde")]
use serde::Serialize;

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Used within [Hcp] to apply handicaps to a vehicle.
pub struct HcpCarHandicap {
    pub added_mass: u8,
    pub intake_restriction: u8,
}

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Vehicle Handicaps
/// You can send a packet to add mass and restrict the intake on each car model
/// The same restriction applies to all drivers using a particular car model
/// This can be useful for creating multi class hosts.
/// The info field is indexed by the vehicle. i.e. XF GTI = 0, XR GT = 1, etc.
pub struct Hcp {
    #[brw(pad_after = 1)]
    pub reqi: RequestId,

    pub info: [HcpCarHandicap; 32],
}
