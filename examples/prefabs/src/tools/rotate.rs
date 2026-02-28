use anyhow::{Result, ensure};
use glam::{DMat2, DVec2};
use insim::{core::heading::Heading, insim::ObjectInfo};

pub fn build(selection: &[ObjectInfo], degrees: f64) -> Result<Vec<ObjectInfo>> {
    ensure!(
        !selection.is_empty(),
        "rotation skipped: selection is empty"
    );
    ensure!(
        degrees.is_finite(),
        "rotation skipped: degrees must be finite"
    );

    let sum: DVec2 = selection
        .iter()
        .map(|obj| obj.position().to_dvec3_metres().truncate())
        .sum();
    let pivot = sum / selection.len() as f64;

    let radians = degrees.to_radians();
    let rotation_mat = DMat2::from_angle(radians);

    let rotated = selection
        .iter()
        .cloned()
        .map(|mut obj| {
            let current_pos = obj.position().to_dvec3_metres().truncate();

            let relative_pos = current_pos - pivot;
            let final_pos = pivot + (rotation_mat * relative_pos);

            let pos = obj.position_mut();
            pos.x = crate::clamp_i16((final_pos.x * 16.0).round() as i32);
            pos.y = crate::clamp_i16((final_pos.y * 16.0).round() as i32);

            if let Some(heading) = obj.heading_mut() {
                *heading = Heading::from_radians(heading.to_radians() + radians);
            }

            obj
        })
        .collect();

    Ok(rotated)
}
