use insim_core::{license::License, vehicle::Vehicle};

pub trait VehicleGameData {
    fn required_license(&self) -> License;
}

impl VehicleGameData for Vehicle {
    fn required_license(&self) -> License {
        match self {
            Vehicle::Xfg => License::Demo,
            Vehicle::Xrg => License::Demo,
            Vehicle::Fbm => License::Demo,

            Vehicle::Xrt => License::S1,
            Vehicle::Rb4 => License::S1,
            Vehicle::Fxo => License::S1,
            Vehicle::Lx4 => License::S1,
            Vehicle::Lx6 => License::S1,
            Vehicle::Mrt => License::S1,

            Vehicle::Uf1 => License::S2,
            Vehicle::Rac => License::S2,
            Vehicle::Fz5 => License::S2,
            Vehicle::Fox => License::S2,
            Vehicle::Xfr => License::S2,
            Vehicle::Ufr => License::S2,
            Vehicle::Fo8 => License::S2,
            Vehicle::Fxr => License::S2,
            Vehicle::Xrr => License::S2,
            Vehicle::Fzr => License::S2,
            Vehicle::Bf1 => License::S2,

            Vehicle::Mod(_) => License::S3,

            _ => {
                panic!("Programming error. Unhandled license")
            }
        }
    }
}
