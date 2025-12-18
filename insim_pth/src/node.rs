//! Node

use glam::{IVec3, Vec3};

use crate::limit::Limit;

/// Node / or point on a track
#[derive(Debug, Copy, Clone, Default, PartialEq, insim_core::Decode, insim_core::Encode)]
pub struct Node {
    /// Center point of this node
    pub center: IVec3,

    /// Expected direction of travel
    pub direction: Vec3,

    /// Track outer limit, relative to the center point and direction of travel
    pub outer_limit: Limit,

    /// Road limit, relative to the center point and direction of travel
    pub road_limit: Limit,
}

impl Node {
    /// Get the center point of this node, optionally scaled
    pub fn get_center(&self, scale: Option<f32>) -> Vec3 {
        let scale = scale.unwrap_or(1.0);

        Vec3 {
            x: self.center.x as f32 / scale,
            y: self.center.y as f32 / scale,
            z: self.center.z as f32 / scale,
        }
    }

    /// Calculate the absolute position of the left and right road limits
    pub fn get_road_limit(&self, scale: Option<f32>) -> (Vec3, Vec3) {
        self.calculate_limit_position(&self.road_limit, scale)
    }

    /// Calculate the absolute position of the left and right track limits
    pub fn get_outer_limit(&self, scale: Option<f32>) -> (Vec3, Vec3) {
        self.calculate_limit_position(&self.outer_limit, scale)
    }

    fn calculate_limit_position(&self, limit: &Limit, scale: Option<f32>) -> (Vec3, Vec3) {
        let center = self.get_center(scale);

        // 90° rotation: (x, y) → (-y, x)
        let left = Vec3 {
            x: -self.direction.y * limit.left + center.x,
            y: self.direction.x * limit.left + center.y,
            z: center.z,
        };

        // -90° rotation: (x, y) → (y, -x)
        let right = Vec3 {
            x: self.direction.y * limit.right + center.x,
            y: -self.direction.x * limit.right + center.y,
            z: center.z,
        };

        (left, right)
    }
}
