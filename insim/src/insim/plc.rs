use std::collections::{hash_set::Iter as HashSetIter, HashSet};

use insim_core::{
    binrw::{self, binrw},
    identifiers::{ConnectionId, RequestId},
    vehicle::Vehicle,
};

#[cfg(feature = "serde")]
use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum PlcAllowedCarsError {
    #[error("Unknown or Mod vehicles cannot be used with the PLC packet, please use MAL")]
    ModInvalid,
}

#[derive(Debug, Default, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct PlcAllowedCars {
    inner: HashSet<Vehicle>,
}

impl PlcAllowedCars {
    const XF_GTI: u32 = (1 << 1);
    const XR_GT: u32 = (1 << 2);
    const XR_GT_TURBO: u32 = (1 << 3);
    const RB4: u32 = (1 << 4);
    const FXO_TURBO: u32 = (1 << 5);
    const LX4: u32 = (1 << 6);
    const LX6: u32 = (1 << 7);
    const MRT5: u32 = (1 << 8);
    const UF_1000: u32 = (1 << 9);
    const RACEABOUT: u32 = (1 << 10);
    const FZ50: u32 = (1 << 11);
    const FORMULA_XR: u32 = (1 << 12);
    const XF_GTR: u32 = (1 << 13);
    const UF_GTR: u32 = (1 << 14);
    const FORMULA_V8: u32 = (1 << 15);
    const FXO_GTR: u32 = (1 << 16);
    const XR_GTR: u32 = (1 << 17);
    const FZ50_GTR: u32 = (1 << 18);
    const BWM_SAUBER_F1_06: u32 = (1 << 19);
    const FORMULA_BMW_FB02: u32 = (1 << 20);

    pub fn contains(&self, v: &Vehicle) -> bool {
        self.inner.contains(v)
    }

    pub fn insert(&mut self, v: Vehicle) -> Result<bool, PlcAllowedCarsError> {
        match v {
            Vehicle::Mod(_) | Vehicle::Unknown => Err(PlcAllowedCarsError::ModInvalid),
            _ => Ok(self.inner.insert(v)),
        }
    }

    pub fn remove(&mut self, v: &Vehicle) -> bool {
        self.inner.remove(v)
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn clear(&mut self) {
        self.inner.clear()
    }

    pub fn iter(&self) -> HashSetIter<'_, Vehicle> {
        self.inner.iter()
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn from_bits_truncate(value: u32) -> Self {
        let mut data = HashSet::default();

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

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Player Cars
pub struct Plc {
    #[brw(pad_after = 1)]
    pub reqi: RequestId,

    #[brw(pad_before = 3)]
    pub ucid: ConnectionId,

    #[br(map = PlcAllowedCars::from_bits_truncate)]
    #[bw(map = |x: &PlcAllowedCars| x.bits())]
    pub allowed_cars: PlcAllowedCars,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hashset_xrg() {
        let mut allowed = PlcAllowedCars::default();

        assert!(allowed.insert(Vehicle::Unknown).is_err());
        assert!(allowed.insert(Vehicle::Mod(1)).is_err());

        allowed.insert(Vehicle::Xfg).unwrap();
        allowed.insert(Vehicle::Xrg).unwrap();

        let reversed =
            PlcAllowedCars::from_bits_truncate(PlcAllowedCars::XR_GT | PlcAllowedCars::XF_GTI);

        assert!(
            reversed.contains(&Vehicle::Xrg)
                && reversed.contains(&Vehicle::Xfg)
                && reversed.len() == 2
        );

        assert_eq!(allowed.bits(), reversed.bits());
    }
}
