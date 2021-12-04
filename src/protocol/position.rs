use deku::prelude::*;
use serde::Serialize;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Serialize)]
pub struct FixedPoint {
    #[deku(bytes = "4")]
    x: i32,

    #[deku(bytes = "4")]
    y: i32,

    #[deku(bytes = "4")]
    z: i32,
}

impl FixedPoint {
    pub fn metres(&self) -> Self {
        FixedPoint {
            x: (self.x / 65536),
            y: (self.y / 65536),
            z: (self.z / 65536),
        }
    }

    pub fn meters(&self) -> Self {
        self.metres()
    }
}
