use insim_core::vehicle::Vehicle;

use crate::identifiers::{PlayerId, RequestId};

#[cfg(feature = "serde")]
mod setup {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub(super) fn ser<T, S>(arr: T, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: IntoIterator,
        T::Item: Serialize,
        S: Serializer,
    {
        serializer.collect_seq(arr)
    }

    pub(super) fn deser<'de, D>(deserializer: D) -> Result<[u8; 120], D::Error>
    where
        D: Deserializer<'de>,
    {
        let vec: Vec<u8> = Vec::deserialize(deserializer)?;
        vec.try_into().map_err(|_| {
            serde::de::Error::custom("failed to deserialize list: expected exactly 120 bytes")
        })
    }
}

#[derive(Debug, Clone, insim_core::Decode, insim_core::Encode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
/// Player sent setup to host
///
/// - Sent when SET is enabled in [IsiFlags](crate::insim::IsiFlags).
pub struct Set {
    /// Request identifier echoed by replies.
    pub reqi: RequestId,

    /// Player that left the race.
    pub plid: PlayerId,

    /// vehicle
    #[insim(pad_after = 4)]
    pub cname: Vehicle,

    /// Fuel Load at start
    #[insim(pad_after = 3)]
    pub fuelload: u8,

    /// Raw setup bytes. Same format as SET file, with the following exceptions:
    /// - less the first 12 bytes
    /// - gear order is the first 7 gears then final drive ratio
    // TODO:: Do we want to strongly type this? I think we should..
    // https://en.lfsmanual.net/wiki/File_Formats#SET
    #[cfg_attr(
        feature = "serde",
        serde(serialize_with = "setup::ser", deserialize_with = "setup::deser")
    )]
    #[cfg_attr(feature = "schemars", schemars(with = "Vec<u8>"))]
    pub setup: [u8; 120],
}

impl Default for Set {
    fn default() -> Self {
        Self {
            reqi: Default::default(),
            plid: Default::default(),
            cname: Default::default(),
            fuelload: Default::default(),
            setup: [0; 120],
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_set() {
        assert_from_to_bytes!(
            Set,
            [
                1, 12, b'X', b'F', b'G', 0, 0, 0, 0, 0, 10, 0, 0, 0, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9,
                10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30,
                31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51,
                52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72,
                73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 91, 92, 93,
                94, 95, 96, 97, 98, 99, 100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111,
                112, 113, 114, 115, 116, 117, 118, 119
            ],
            |parsed: Set| {
                assert_eq!(parsed.reqi, RequestId(1));
                assert_eq!(parsed.plid, PlayerId(12));
                assert_eq!(parsed.cname, Vehicle::Xfg);
                assert_eq!(parsed.fuelload, 10);
                assert_eq!(
                    parsed.setup,
                    [
                        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
                        21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39,
                        40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58,
                        59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77,
                        78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 91, 92, 93, 94, 95, 96,
                        97, 98, 99, 100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111,
                        112, 113, 114, 115, 116, 117, 118, 119
                    ]
                )
            }
        );
    }
}
