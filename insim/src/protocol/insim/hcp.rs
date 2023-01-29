use bytes::BytesMut;
use insim_core::{identifiers::RequestId, prelude::*, ser::Limit, DecodableError, EncodableError};

#[cfg(feature = "serde")]
use serde::Serialize;

#[derive(Debug, InsimEncode, InsimDecode, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Used within [Hcp] to apply handicaps to a vehicle.
pub struct HcpCarHandicap {
    pub added_mass: u8,
    pub intake_restriction: u8,
}

#[derive(Debug, Clone, Default)]
// we need to wrap [PlayerId; 32] in a new type because arrays are always considered "foreign", and the trait Decodable isn't defined within this crate.
// FIXME: add some extra methods for convenience
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct HcpCarHandicapList([HcpCarHandicap; 32]);

#[derive(Debug, InsimEncode, InsimDecode, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Vehicle Handicaps
/// You can send a packet to add mass and restrict the intake on each car model
/// The same restriction applies to all drivers using a particular car model
/// This can be useful for creating multi class hosts.
/// The info field is indexed by the vehicle. i.e. XF GTI = 0, XR GT = 1, etc.
pub struct Hcp {
    #[insim(pad_bytes_after = "1")]
    pub reqi: RequestId,

    pub info: HcpCarHandicapList,
}

impl Decodable for HcpCarHandicapList {
    fn decode(buf: &mut BytesMut, limit: Option<Limit>) -> Result<Self, DecodableError> {
        if limit.is_some() {
            return Err(DecodableError::UnexpectedLimit(format!(
                "HcpCarHandicap does not support a limit: {:?}",
                limit
            )));
        }

        let mut data: HcpCarHandicapList = Default::default();
        for i in 0..32 {
            data.0[i] = HcpCarHandicap::decode(buf, None)?;
        }

        Ok(data)
    }
}

impl Encodable for HcpCarHandicapList {
    fn encode(&self, buf: &mut BytesMut, limit: Option<Limit>) -> Result<(), EncodableError>
    where
        Self: Sized,
    {
        if limit.is_some() {
            return Err(EncodableError::UnexpectedLimit(format!(
                "HcpCarHandicap does not support a limit: {:?}",
                limit
            )));
        }

        for i in self.0.iter() {
            i.encode(buf, None)?;
        }

        Ok(())
    }
}
