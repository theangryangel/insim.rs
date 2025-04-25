use insim_core::binrw::{self, binrw};

use crate::identifiers::RequestId;

#[binrw]
#[derive(Debug, Clone, Default, insim_macros::ReadWriteBuf)]
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
#[derive(Debug, Clone, Default, insim_macros::ReadWriteBuf)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Vehicle Handicaps
/// You can send a packet to add mass and restrict the intake on each car model
/// The same restriction applies to all drivers using a particular car model
/// This can be useful for creating multi class hosts.
/// The info field is indexed by the vehicle. i.e. XF GTI = 0, XR GT = 1, etc.
pub struct Hcp {
    #[brw(pad_after = 1)]
    #[read_write_buf(pad_after = 1)]
    /// Non-zero if the packet is a packet request or a reply to a request
    pub reqi: RequestId,

    /// H_Mass and H_TRes for each car: : XF GTI = 0 / XR GT = 1 etc
    // TODO: make this work with [Vehicle]
    pub info: [HcpCarHandicap; 32],
}

impl_typical_with_request_id!(Hcp);

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_hcp() {
        assert_from_to_bytes!(
            Hcp,
            [
                0,  // reqi
                0,  // zero
                10, // carhcp[0] - h_mass
                25, // carhcp[0] - h_tres
                10, // carhcp[1] - h_mass
                25, // carhcp[1] - h_tres
                10, // carhcp[2] - h_mass
                25, // carhcp[2] - h_tres
                10, // carhcp[3] - h_mass
                25, // carhcp[3] - h_tres
                10, // carhcp[4] - h_mass
                25, // carhcp[4] - h_tres
                10, // carhcp[5] - h_mass
                25, // carhcp[5] - h_tres
                10, // carhcp[6] - h_mass
                25, // carhcp[6] - h_tres
                10, // carhcp[7] - h_mass
                25, // carhcp[7] - h_tres
                10, // carhcp[8] - h_mass
                25, // carhcp[8] - h_tres
                10, // carhcp[9] - h_mass
                25, // carhcp[9] - h_tres
                10, // carhcp[10] - h_mass
                25, // carhcp[10] - h_tres
                10, // carhcp[11] - h_mass
                25, // carhcp[11] - h_tres
                10, // carhcp[12] - h_mass
                25, // carhcp[12] - h_tres
                10, // carhcp[13] - h_mass
                25, // carhcp[13] - h_tres
                10, // carhcp[14] - h_mass
                25, // carhcp[14] - h_tres
                10, // carhcp[15] - h_mass
                25, // carhcp[15] - h_tres
                10, // carhcp[16] - h_mass
                25, // carhcp[16] - h_tres
                10, // carhcp[17] - h_mass
                25, // carhcp[17] - h_tres
                10, // carhcp[18] - h_mass
                25, // carhcp[18] - h_tres
                10, // carhcp[19] - h_mass
                25, // carhcp[19] - h_tres
                10, // carhcp[20] - h_mass
                25, // carhcp[20] - h_tres
                10, // carhcp[21] - h_mass
                25, // carhcp[21] - h_tres
                10, // carhcp[22] - h_mass
                25, // carhcp[22] - h_tres
                10, // carhcp[23] - h_mass
                25, // carhcp[23] - h_tres
                10, // carhcp[24] - h_mass
                25, // carhcp[24] - h_tres
                10, // carhcp[25] - h_mass
                25, // carhcp[25] - h_tres
                10, // carhcp[26] - h_mass
                25, // carhcp[26] - h_tres
                10, // carhcp[27] - h_mass
                25, // carhcp[27] - h_tres
                10, // carhcp[28] - h_mass
                25, // carhcp[28] - h_tres
                10, // carhcp[29] - h_mass
                25, // carhcp[29] - h_tres
                10, // carhcp[30] - h_mass
                25, // carhcp[30] - h_tres
                11, // carhcp[31] - h_mass
                26, // carhcp[31] - h_tres
            ],
            |hcp: Hcp| {
                assert_eq!(hcp.reqi, RequestId(0));
                assert_eq!(hcp.info[1].h_tres, 25);
            }
        );
    }
}
