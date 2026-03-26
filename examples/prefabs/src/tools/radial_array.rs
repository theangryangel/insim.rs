use anyhow::{Result, ensure};
use glam::{DMat2, DVec2};
use insim::{core::heading::Heading, insim::ObjectInfo};

/// Arrange `count` copies of `selection` evenly around a circle.
///
/// The selection's centroid is the centre of the circle. Each copy is placed
/// `radius_metres` away from the centroid, with the first copy offset in the
/// +Y direction. Copies are spaced `total_arc_degrees / count` apart, with
/// object headings updated to match.
pub fn build(
    selection: &[ObjectInfo],
    count: usize,
    radius_metres: f64,
    total_arc_degrees: f64,
) -> Result<Vec<ObjectInfo>> {
    ensure!(
        !selection.is_empty(),
        "radial array skipped: selection is empty"
    );
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

    let angle_step_rad = total_arc_degrees.to_radians() / count as f64;

    // Each object's offset from the centroid, rotated and then placed at
    // radius distance from the centroid.
    let mut result = Vec::with_capacity(count * selection.len());

    for k in 0..count {
        let angle = k as f64 * angle_step_rad;
        let rot = DMat2::from_angle(angle);

        // Radial offset for this copy: start at +Y then rotate.
        let radial_offset = rot * DVec2::new(0.0, radius_metres);

        for obj in selection {
            let mut new_obj = obj.clone();

            // Preserve the object's position relative to the centroid,
            // then place the whole group at the radial offset.
            let world_pos = obj.position().to_dvec3_metres().truncate();
            let local = world_pos - centroid;
            let placed = centroid + radial_offset + rot * local;

            let p = new_obj.position_mut();
            p.x = crate::clamp_i16((placed.x * 16.0).round() as i32);
            p.y = crate::clamp_i16((placed.y * 16.0).round() as i32);

            if let Some(heading) = new_obj.heading_mut() {
                *heading = Heading::from_radians(heading.to_radians() + angle);
            }

            result.push(new_obj);
        }
    }

    Ok(result)
}
