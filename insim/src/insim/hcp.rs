use insim_core::vehicle::Vehicle;

use crate::identifiers::RequestId;

#[derive(Debug, Clone, Default, insim_core::Decode, insim_core::Encode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Handicap settings for a single vehicle.
pub struct HcpCarHandicap {
    /// Added mass (0-200 kg).
    pub h_mass: u8,

    /// Intake restriction (0-50).
    pub h_tres: u8,
}

#[derive(Debug, Clone, Default, insim_core::Decode, insim_core::Encode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Per-vehicle handicap settings.
///
/// - Applies mass and intake restrictions per car model.
/// - Use [`set`](Hcp::set) and [`get`](Hcp::get) for [Vehicle] indexing.
pub struct Hcp {
    #[insim(pad_after = 1)]
    /// Request identifier echoed by replies.
    pub reqi: RequestId,

    /// Handicaps for each car model (indexed by [Vehicle]).
    pub info: [HcpCarHandicap; 32],
}

impl Hcp {
    fn lookup_vehicle(vehicle: Vehicle) -> crate::Result<usize> {
        let index = match vehicle {
            Vehicle::Xfg => 0,
            Vehicle::Xrg => 1,
            Vehicle::Fbm => 20,
            Vehicle::Xrt => 3,
            Vehicle::Rb4 => 4,
            Vehicle::Fxo => 5,
            Vehicle::Lx4 => 6,
            Vehicle::Lx6 => 7,
            Vehicle::Mrt => 8,
            Vehicle::Uf1 => 9,
            Vehicle::Rac => 10,
            Vehicle::Fz5 => 11,
            Vehicle::Fox => 12,
            Vehicle::Xfr => 13,
            Vehicle::Ufr => 14,
            Vehicle::Fo8 => 15,
            Vehicle::Fxr => 16,
            Vehicle::Xrr => 17,
            Vehicle::Fzr => 18,
            Vehicle::Bf1 => 19,
            _ => return Err(crate::Error::VehicleNotStandard),
        };

        Ok(index)
    }

    /// Set the handicaps for a given [Vehicle]
    pub fn set(&mut self, vehicle: Vehicle, h_mass: u8, h_tres: u8) -> crate::Result<()> {
        let index = Self::lookup_vehicle(vehicle)?;
        self.info[index].h_tres = h_tres;
        self.info[index].h_mass = h_mass;
        Ok(())
    }

    /// Retreive the current handicap for a given [Vehicle]
    pub fn get(&self, vehicle: Vehicle) -> crate::Result<&HcpCarHandicap> {
        let index = Self::lookup_vehicle(vehicle)?;
        Ok(&self.info[index])
    }
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
