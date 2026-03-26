use anyhow::{Result, ensure};
use glam::DVec2;
use insim::{core::heading::Heading, insim::ObjectInfo};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MirrorAxis {
    /// Flip across the X axis: negates each object's Y offset from the centroid.
    /// Left↔right swap. New heading h' = π − h.
    X,
    /// Flip across the Y axis: negates each object's X offset from the centroid.
    /// Front↔back swap. New heading h' = −h.
    Y,
}

pub fn build(selection: &[ObjectInfo], axis: MirrorAxis) -> Result<Vec<ObjectInfo>> {
    ensure!(!selection.is_empty(), "mirror skipped: selection is empty");

    let sum: DVec2 = selection
        .iter()
        .map(|obj| obj.position().to_dvec3_metres().truncate())
        .sum();
    let centroid = sum / selection.len() as f64;

    let mirrored = selection
        .iter()
        .cloned()
        .map(|mut obj| {
            let pos = obj.position().to_dvec3_metres().truncate();
            let rel = pos - centroid;

            let new_rel = match axis {
                MirrorAxis::X => DVec2::new(rel.x, -rel.y),
                MirrorAxis::Y => DVec2::new(-rel.x, rel.y),
            };
            let final_pos = centroid + new_rel;

            let p = obj.position_mut();
            p.x = crate::clamp_i16((final_pos.x * 16.0).round() as i32);
            p.y = crate::clamp_i16((final_pos.y * 16.0).round() as i32);

            if let Some(heading) = obj.heading_mut() {
                let h = heading.to_radians();
                let new_h = match axis {
                    MirrorAxis::X => std::f64::consts::PI - h,
                    MirrorAxis::Y => -h,
                };
                *heading = Heading::from_radians(new_h);
            }

            obj
        })
        .collect();

    Ok(mirrored)
}
