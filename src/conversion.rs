pub mod speed {
    /// Convert a LFS speed to km/h
    pub fn to_kmph(speed: u16) -> f32 {
        (speed as f32) / 91.02
    }

    /// Convert a LFS speed to Miles/h
    pub fn to_mph(speed: u16) -> f32 {
        (speed as f32) / 146.48607
    }

    /// Convert a LFS speed to Metres/sec
    pub fn to_mps(speed: u16) -> f32 {
        (speed as f32) / 327.68
    }
}

pub mod distance {
    /// Convert a LFS distance to metres
    pub fn to_metres(distance: u16) -> f32 {
        (distance as f32) / 65536.0
    }

    /// An Amercanised version of to_metres
    pub fn to_meters(distance: u16) -> f32 {
        to_metres(distance)
    }

    /// Convert LFS distance to km
    pub fn to_km(distance: u16) -> f32 {
        (distance as f32) / 65536.0 / 1000.0
    }

    /// Convert LFS distance to miles
    pub fn to_miles(distance: u16) -> f32 {
        (distance as f32) / 65536.0 / 1609.344
    }
}
