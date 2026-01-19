use std::default::Default;

use bytes::{Buf, BufMut};
use indexmap::{IndexSet, set::Iter as IndexSetIter};
use insim_core::{Decode, Encode, vehicle::Vehicle};

use crate::{
    error::Error,
    identifiers::{ConnectionId, RequestId},
};

const MAX_MAL_SIZE: usize = 120;

#[derive(Debug, Clone, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Mods Allowed - restrict the mods that can be used
pub struct Mal {
    /// Non-zero if the packet is a packet request or a reply to a request
    pub reqi: RequestId,

    /// UCID to change
    pub ucid: ConnectionId,

    allowed_mods: IndexSet<Vehicle>,
}

impl Mal {
    /// Returns `true` if a Vehicle is contained in this packet
    pub fn contains(&self, v: &Vehicle) -> bool {
        self.allowed_mods.contains(v)
    }

    /// Push a compressed form of a mod onto the list of allowed mods
    /// and update the count.
    pub fn insert(&mut self, vehicle: Vehicle) -> Result<bool, Error> {
        match vehicle {
            Vehicle::Mod(_) => Ok(self.allowed_mods.insert(vehicle)),
            _ => Err(Error::VehicleNotAMod),
        }
    }

    /// Remove a Vehicle from this packet
    pub fn remove(&mut self, vehicle: &Vehicle) -> bool {
        self.allowed_mods.shift_remove(vehicle)
    }

    /// Does this packet have no vehicles associated?
    pub fn is_empty(&self) -> bool {
        self.allowed_mods.is_empty()
    }

    /// Clear any previously allowed mods.
    pub fn clear(&mut self) {
        self.allowed_mods.clear()
    }

    /// Iterator for all allowed mods
    pub fn iter(&self) -> IndexSetIter<'_, Vehicle> {
        self.allowed_mods.iter()
    }

    /// Returns the number of allowed mods
    pub fn len(&self) -> usize {
        self.allowed_mods.len()
    }
}

impl Decode for Mal {
    fn decode(buf: &mut bytes::Bytes) -> Result<Self, insim_core::DecodeError> {
        let reqi = RequestId::decode(buf)?;
        let mut numm = u8::decode(buf)?;
        let ucid = ConnectionId::decode(buf)?;
        buf.advance(3);
        let mut set = IndexSet::with_capacity(numm as usize);

        while numm > 0 {
            let _ = set.insert(Vehicle::Mod(u32::decode(buf)?));
            numm -= 1;
        }

        Ok(Self {
            reqi,
            ucid,
            allowed_mods: set,
        })
    }
}

impl Encode for Mal {
    fn encode(&self, buf: &mut bytes::BytesMut) -> Result<(), insim_core::EncodeError> {
        self.reqi.encode(buf)?;
        if self.allowed_mods.len() > MAX_MAL_SIZE {
            return Err(insim_core::EncodeErrorKind::OutOfRange { min: 0, max: MAX_MAL_SIZE, found: self.allowed_mods.len() }.context("Mal allowed_mods out of range"));
        }
        (self.allowed_mods.len() as u8).encode(buf)?;
        self.ucid.encode(buf)?;
        buf.put_bytes(0, 3);
        for i in self.allowed_mods.iter() {
            match i {
                Vehicle::Mod(ident) => ident.encode(buf)?,
                _ => unreachable!(
                    "Non-Mod vehicle managed to get into the HashSet. Should not be possible."
                ),
            }
        }

        Ok(())
    }
}

impl_typical_with_request_id!(Mal);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standard_vehicle_rejected() {
        let mut data = Mal::default();

        assert!(data.insert(Vehicle::Xfg).is_err());
        assert!(data.insert(Vehicle::Mod(1)).is_ok());
        assert_eq!(data.len(), 1);
    }

    #[test]
    fn test_mal() {
        assert_from_to_bytes!(
            Mal,
            [
                2, // reqi
                2, // numm
                3, // connection id
                0, 0, 0, // padding / unused
                1, 0, 0, 0, // mod 1
                2, 0, 0, 0, // mod 2
            ],
            |mal: Mal| {
                assert_eq!(mal.reqi, RequestId(2));
                assert_eq!(mal.ucid, ConnectionId(3));
                assert_eq!(mal.len(), 2);
                assert!(mal.contains(&Vehicle::Mod(1)));
                assert!(mal.contains(&Vehicle::Mod(2)));
            }
        );
    }
}
