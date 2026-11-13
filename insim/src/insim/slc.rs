use insim_core::vehicle::Vehicle;

use crate::identifiers::{ConnectionId, RequestId};

#[derive(Debug, Clone, Default, insim_core::Decode, insim_core::Encode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Connection selected a car.
///
/// - Sent when a connection selects a car.
pub struct Slc {
    /// Request identifier echoed by replies.
    pub reqi: RequestId,

    /// Connection that selected the car.
    pub ucid: ConnectionId,

    /// Selected vehicle.
    pub cname: Vehicle,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_slc_xrt() {
        assert_from_to_bytes!(
            Slc,
            [
                3,  // reqi
                2,  // ucid
                88, // cname (1)
                82, // cname (2)
                84, // cname (3)
                0,  // cname (4)
            ],
            |slc: Slc| {
                assert_eq!(slc.reqi, RequestId(3));
                assert_eq!(slc.ucid, ConnectionId(2));
                assert_eq!(slc.cname, Vehicle::Xrt);
            }
        )
    }

    #[test]
    fn test_slc_mod() {
        assert_from_to_bytes!(
            Slc,
            [
                3,   // reqi
                2,   // ucid
                230, // cname (1)
                130, // cname (2)
                88,  // cname (3)
                0,   // cname (4)
            ],
            |slc: Slc| {
                assert_eq!(slc.reqi, RequestId(3));
                assert_eq!(slc.ucid, ConnectionId(2));
                assert_eq!(slc.cname, Vehicle::Mod(0x5882E6));
            }
        )
    }
}
