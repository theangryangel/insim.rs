use crate::identifiers::{PlayerId, RequestId};

#[derive(Debug, Clone, Default, insim_core::Decode, insim_core::Encode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "python", pyo3::prelude::pyclass)]
/// Car Reset packet indicates a vehicle has been reset or that a vehicle should be reset by the
/// server.
pub struct Crs {
    /// Non-zero if the packet is a packet request or a reply to a request
    pub reqi: RequestId,
    /// Unique player ID that was reset, or should be reset
    pub plid: PlayerId,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_crs() {
        assert_from_to_bytes!(Crs, [1, 9], |crs: Crs| {
            assert_eq!(crs.reqi, RequestId(1));
            assert_eq!(crs.plid, PlayerId(9));
        });
    }
}
