//! Object Position

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Layout Object Position
pub struct ObjectCoordinate {
    /// X coordinate (1:16 scale)
    pub x: i16,
    /// Y coordinate (1:16 scale)
    pub y: i16,
    /// X coordinate (1:4 scale)
    pub z: u8,
}

impl ObjectCoordinate {
    // Scale to metres for X and Y
    const SCALE: i16 = 16;

    /// X (in metres)
    pub fn x_metres(&self) -> f32 {
        self.x as f32 / Self::SCALE as f32
    }

    /// Y (in metres)
    pub fn y_metres(&self) -> f32 {
        self.y as f32 / Self::SCALE as f32
    }

    /// Z (in metres)
    pub fn z_metres(&self) -> f32 {
        self.z as f32 / 4.0
    }

    /// X, Y, Z (in metres)
    pub fn xyz_metres(&self) -> (f32, f32, f32) {
        (self.x_metres(), self.y_metres(), self.z_metres())
    }

    /// New ObjectCoordinate in raw units
    pub fn new(x: i16, y: i16, z: u8) -> Self {
        Self { x, y, z }
    }
}

#[cfg(feature = "glam")]
impl ObjectCoordinate {
    /// Convert to glam Vec3, where xyz are in raw
    pub fn to_ivec3(&self) -> glam::I16Vec3 {
        glam::I16Vec3 {
            x: self.x,
            y: self.y,
            z: self.z as i16,
        }
    }

    /// Convert from glam IVec3, where xyz are in raw
    pub fn from_ivec3(other: glam::I16Vec3) -> Self {
        Self {
            x: other.x,
            y: other.y,
            z: other.z as u8,
        }
    }

    /// Convert to glam DVec3, where xyz are in metres
    pub fn to_dvec3_metres(&self) -> glam::DVec3 {
        glam::DVec3 {
            x: (self.x as f64 / 16.0),
            y: (self.y as f64 / 16.0),
            z: (self.z as f64 / 4.0),
        }
    }

    /// Convert from glam DVec3, where xyz are in metres
    pub fn from_dvec3_metres(other: glam::DVec3) -> Self {
        Self {
            x: (other.x * 16.0).round() as i16,
            y: (other.y * 16.0).round() as i16,
            z: (other.z * 4.0).round() as u8,
        }
    }

    /// Convert to glam Vec3, where xyz are in metres
    pub fn to_vec3_metres(&self) -> glam::Vec3 {
        glam::Vec3 {
            x: (self.x as f32 / 16.0),
            y: (self.y as f32 / 16.0),
            z: (self.z as f32 / 4.0),
        }
    }

    /// Convert from glam Vec3, where xyz are in metres
    pub fn from_vec3_metres(other: glam::Vec3) -> Self {
        Self {
            x: (other.x * 16.0).round() as i16,
            y: (other.y * 16.0).round() as i16,
            z: (other.z * 4.0).round() as u8,
        }
    }
}
