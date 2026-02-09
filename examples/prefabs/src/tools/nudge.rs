use glam::DVec3;
use insim::{
    core::{heading::Heading, object::ObjectCoordinate},
    insim::ObjectInfo,
};

pub fn nudge(selection: &[ObjectInfo], heading: Heading, distance_metres: f64) -> Vec<ObjectInfo> {
    let mut output = Vec::with_capacity(selection.len());
    let rads = heading.to_radians();
    let translation = DVec3::new(rads.sin(), -rads.cos(), 0.0) * distance_metres;
    for obj in selection {
        let mut new_obj = obj.clone();
        let current_pos = obj.position().to_dvec3_metres();
        let new_pos = current_pos + translation;
        *new_obj.position_mut() = ObjectCoordinate::from_dvec3_metres(new_pos);
        output.push(new_obj);
    }
    output
}
