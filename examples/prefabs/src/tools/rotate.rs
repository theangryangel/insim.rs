use anyhow::{Result, anyhow};
use glam::DVec2;
use insim::{core::heading::Heading, insim::ObjectInfo};

pub fn build(selection: &[ObjectInfo], degrees: f64) -> Result<Vec<ObjectInfo>> {
    if selection.is_empty() {
        return Err(anyhow!("rotation skipped: selection is empty"));
    }
    if !degrees.is_finite() {
        return Err(anyhow!("rotation skipped: degrees must be finite"));
    }

    let len = selection.len() as f64;
    let sum = selection.iter().fold(DVec2::ZERO, |acc, obj| {
        let pos = obj.position().to_dvec3_metres();
        acc + DVec2::new(pos.x, pos.y)
    });
    let pivot = sum / len;

    let radians = degrees.to_radians();
    let cos_theta = radians.cos();
    let sin_theta = radians.sin();

    let rotated = selection
        .iter()
        .cloned()
        .map(|mut obj| {
            let pos = obj.position().to_dvec3_metres();
            let current = DVec2::new(pos.x, pos.y);
            let delta = current - pivot;

            let rotated = DVec2::new(
                delta.x * cos_theta - delta.y * sin_theta,
                delta.x * sin_theta + delta.y * cos_theta,
            );
            let final_pos = pivot + rotated;

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
