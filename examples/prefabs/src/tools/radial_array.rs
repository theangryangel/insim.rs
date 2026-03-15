use anyhow::{Result, ensure};
use glam::{DMat2, DVec2};
use insim::{core::heading::Heading, insim::ObjectInfo};

/// Arrange `count` copies of `selection` evenly around a circle.
///
/// The array centre is placed `radius_metres` in the +Y direction from the
/// selection's centroid, so the first copy stays in its original position.
/// Each subsequent copy is rotated around that centre by
/// `total_arc_degrees / count` per step, with object headings updated to
/// match.
pub fn build(
    selection: &[ObjectInfo],
    count: usize,
    radius_metres: f64,
    total_arc_degrees: f64,
) -> Result<Vec<ObjectInfo>> {
    ensure!(!selection.is_empty(), "radial array skipped: selection is empty");
    ensure!(count >= 2, "radial array skipped: count must be at least 2");
    ensure!(
        radius_metres.is_finite() && radius_metres > 0.0,
        "radial array skipped: radius must be a positive finite number"
    );
    ensure!(
        total_arc_degrees.is_finite() && total_arc_degrees != 0.0,
        "radial array skipped: arc must be a non-zero finite number"
    );

    let sum: DVec2 = selection
        .iter()
        .map(|obj| obj.position().to_dvec3_metres().truncate())
        .sum();
    let centroid = sum / selection.len() as f64;

    // The pivot is offset from the centroid along +Y by the radius so that
    // copy 0 sits at angle 0 (its original position) and copies fan outward.
    let pivot = centroid + DVec2::new(0.0, radius_metres);

    let angle_step_rad = total_arc_degrees.to_radians() / count as f64;

    let mut result = Vec::with_capacity(count * selection.len());

    for k in 0..count {
        let angle = k as f64 * angle_step_rad;
        let rot = DMat2::from_angle(angle);

        for obj in selection {
            let mut new_obj = obj.clone();

            let world_pos = obj.position().to_dvec3_metres().truncate();
            let relative = world_pos - pivot;
            let rotated = pivot + rot * relative;

            let p = new_obj.position_mut();
            p.x = crate::clamp_i16((rotated.x * 16.0).round() as i32);
            p.y = crate::clamp_i16((rotated.y * 16.0).round() as i32);

            if let Some(heading) = new_obj.heading_mut() {
                *heading = Heading::from_radians(heading.to_radians() + angle);
            }

            result.push(new_obj);
        }
    }

    Ok(result)
}
