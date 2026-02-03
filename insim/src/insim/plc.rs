use bytes::{Buf, BufMut};
use indexmap::{IndexSet, set::Iter as IndexSetIter};
use insim_core::{Decode, Encode, vehicle::Vehicle};

use crate::{
    error::Error,
    identifiers::{ConnectionId, RequestId},
};

#[derive(Debug, Default, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Set of allowed standard vehicles for [Plc].
pub struct PlcAllowedCarsSet {
    inner: IndexSet<Vehicle>,
}

impl PlcAllowedCarsSet {
    const XF_GTI: u32 = 1;
    const XR_GT: u32 = (1 << 1);
    const XR_GT_TURBO: u32 = (1 << 2);
    const RB4: u32 = (1 << 3);
    const FXO_TURBO: u32 = (1 << 4);
    const LX4: u32 = (1 << 5);
    const LX6: u32 = (1 << 6);
    const MRT5: u32 = (1 << 7);
    const UF_1000: u32 = (1 << 8);
    const RACEABOUT: u32 = (1 << 9);
    const FZ50: u32 = (1 << 10);
    const FORMULA_XR: u32 = (1 << 11);
    const XF_GTR: u32 = (1 << 12);
    const UF_GTR: u32 = (1 << 13);
    const FORMULA_V8: u32 = (1 << 14);
    const FXO_GTR: u32 = (1 << 15);
    const XR_GTR: u32 = (1 << 16);
    const FZ50_GTR: u32 = (1 << 17);
    const BWM_SAUBER_F1_06: u32 = (1 << 18);
    const FORMULA_BMW_FB02: u32 = (1 << 19);

    /// Does this set include a vehicle?
    pub fn contains(&self, v: &Vehicle) -> bool {
        self.inner.contains(v)
    }

    /// Attempt to add a vehicle.
    pub fn insert(&mut self, v: Vehicle) -> Result<bool, Error> {
        match v {
            Vehicle::Mod(_) | Vehicle::Unknown => Err(Error::VehicleNotStandard),
            _ => Ok(self.inner.insert(v)),
        }
    }

    /// Remove a vehicle, if it's included in the set.
    pub fn remove(&mut self, v: &Vehicle) -> bool {
        self.inner.shift_remove(v)
    }

    /// Is this set empty?
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Clear the set.
    pub fn clear(&mut self) {
        self.inner.clear()
    }

    /// Iterate through the set.
    pub fn iter(&self) -> IndexSetIter<'_, Vehicle> {
        self.inner.iter()
    }

    /// Number of items in the set.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Transform the network representation of the set into [PlcAllowedCarsSet].
    #[allow(unused_results)]
    pub fn from_bits_truncate(value: u32) -> Self {
        let mut data = IndexSet::default();

        if (value & Self::XF_GTI) == Self::XF_GTI {
            data.insert(Vehicle::Xfg);
        }
        if (value & Self::XR_GT) == Self::XR_GT {
            data.insert(Vehicle::Xrg);
        }
        if (value & Self::FORMULA_BMW_FB02) == Self::FORMULA_BMW_FB02 {
            data.insert(Vehicle::Fbm);
        }
        if (value & Self::XR_GT_TURBO) == Self::XR_GT_TURBO {
            data.insert(Vehicle::Xrt);
        }
        if (value & Self::RB4) == Self::RB4 {
            data.insert(Vehicle::Rb4);
        }
        if (value & Self::FXO_TURBO) == Self::FXO_TURBO {
            data.insert(Vehicle::Fxo);
        }
        if (value & Self::LX4) == Self::LX4 {
            data.insert(Vehicle::Lx4);
        }
        if (value & Self::LX6) == Self::LX6 {
            data.insert(Vehicle::Lx6);
        }
        if (value & Self::MRT5) == Self::MRT5 {
            data.insert(Vehicle::Mrt);
        }
        if (value & Self::UF_1000) == Self::UF_1000 {
            data.insert(Vehicle::Uf1);
        }
        if (value & Self::RACEABOUT) == Self::RACEABOUT {
            data.insert(Vehicle::Rac);
        }
        if (value & Self::FZ50) == Self::FZ50 {
            data.insert(Vehicle::Fz5);
        }
        if (value & Self::FORMULA_XR) == Self::FORMULA_XR {
            data.insert(Vehicle::Fox);
        }
        if (value & Self::XF_GTR) == Self::XF_GTR {
            data.insert(Vehicle::Xfr);
        }
        if (value & Self::UF_GTR) == Self::UF_GTR {
            data.insert(Vehicle::Ufr);
        }
        if (value & Self::FORMULA_V8) == Self::FORMULA_V8 {
            data.insert(Vehicle::Fo8);
        }
        if (value & Self::FXO_GTR) == Self::FXO_GTR {
            data.insert(Vehicle::Fxr);
        }
        if (value & Self::XR_GTR) == Self::XR_GTR {
            data.insert(Vehicle::Xrr);
        }
        if (value & Self::FZ50_GTR) == Self::FZ50_GTR {
            data.insert(Vehicle::Fzr);
        }
        if (value & Self::BWM_SAUBER_F1_06) == Self::BWM_SAUBER_F1_06 {
            data.insert(Vehicle::Bf1);
        }

        Self { inner: data }
    }

    /// Output the network representation of this [PlcAllowedCarsSet].
    pub fn bits(&self) -> u32 {
        let mut data: u32 = 0;

        for i in self.inner.iter() {
            data |= match i {
                Vehicle::Xfg => Self::XF_GTI,
                Vehicle::Xrg => Self::XR_GT,
                Vehicle::Fbm => Self::FORMULA_BMW_FB02,
                Vehicle::Xrt => Self::XR_GT_TURBO,
                Vehicle::Rb4 => Self::RB4,
                Vehicle::Fxo => Self::FXO_TURBO,
                Vehicle::Lx4 => Self::LX4,
                Vehicle::Lx6 => Self::LX6,
                Vehicle::Mrt => Self::MRT5,
                Vehicle::Uf1 => Self::UF_1000,
                Vehicle::Rac => Self::RACEABOUT,
                Vehicle::Fz5 => Self::FZ50,
                Vehicle::Fox => Self::FORMULA_XR,
                Vehicle::Xfr => Self::XF_GTR,
                Vehicle::Ufr => Self::UF_GTR,
                Vehicle::Fo8 => Self::FORMULA_V8,
                Vehicle::Fxr => Self::FXO_GTR,
                Vehicle::Xrr => Self::XR_GTR,
                Vehicle::Fzr => Self::FZ50_GTR,
                Vehicle::Bf1 => Self::BWM_SAUBER_F1_06,
                _ => 0,
            }
        }

        data
    }
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Restrict which standard vehicles a connection may select.
///
/// - Applies to standard (non-mod) vehicles only.
pub struct Plc {
    /// Request identifier echoed by replies.
    pub reqi: RequestId,

    /// Connection to restrict.
    pub ucid: ConnectionId,

    /// Allowed vehicle set.
    pub cars: PlcAllowedCarsSet,
}

impl_typical_with_request_id!(Plc);

impl Decode for Plc {
    fn decode(buf: &mut bytes::Bytes) -> Result<Self, insim_core::DecodeError> {
        let reqi = RequestId::decode(buf).map_err(|e| e.nested().context("Plc::reqi"))?;
        buf.advance(1);
        let ucid = ConnectionId::decode(buf).map_err(|e| e.nested().context("Plc::ucid"))?;
        buf.advance(3);
        let cars = PlcAllowedCarsSet::from_bits_truncate(
            u32::decode(buf).map_err(|e| e.nested().context("Plc::cars"))?,
        );
        Ok(Self { reqi, ucid, cars })
    }
}

impl Encode for Plc {
    fn encode(&self, buf: &mut bytes::BytesMut) -> Result<(), insim_core::EncodeError> {
        self.reqi
            .encode(buf)
            .map_err(|e| e.nested().context("Plc::reqi"))?;
        buf.put_bytes(0, 1);
        self.ucid
            .encode(buf)
            .map_err(|e| e.nested().context("Plc::ucid"))?;
        buf.put_bytes(0, 3);
        self.cars
            .bits()
            .encode(buf)
            .map_err(|e| e.nested().context("Plc::cars"))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plcallowedcarsset() {
        let mut allowed = PlcAllowedCarsSet::default();

        assert!(allowed.insert(Vehicle::Unknown).is_err());
        assert!(allowed.insert(Vehicle::Mod(1)).is_err());

        let _ = allowed.insert(Vehicle::Xfg).unwrap();
        let _ = allowed.insert(Vehicle::Xrg).unwrap();

        let reversed = PlcAllowedCarsSet::from_bits_truncate(
            PlcAllowedCarsSet::XR_GT | PlcAllowedCarsSet::XF_GTI,
        );

        assert!(
            reversed.contains(&Vehicle::Xrg)
                && reversed.contains(&Vehicle::Xfg)
                && reversed.len() == 2
        );

        assert_eq!(allowed.bits(), reversed.bits());
    }

    #[test]
    fn test_plc() {
        assert_from_to_bytes!(
            Plc,
            [
                0,  // reqi
                0,  // zero
                13, // ucid
                0,  // sp1
                0,  // sp2
                0,  // sp3
                68, // carflags (1)
                8,  // carflags (2)
                0,  // carflags (3)
                0,  // carflags (4)
            ],
            |parsed: Plc| {
                assert_eq!(parsed.ucid, ConnectionId(13));
                assert!(
                    parsed.cars.contains(&Vehicle::Fox)
                        && parsed.cars.contains(&Vehicle::Lx6)
                        && parsed.cars.contains(&Vehicle::Xrt)
                );
            }
        );
    }
}
