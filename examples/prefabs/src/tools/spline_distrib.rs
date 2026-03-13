use anyhow::{Result, anyhow};
use glam::DVec3;
use insim::{
    core::{heading::Heading, object::ObjectCoordinate},
    insim::ObjectInfo,
};

use super::spline;

pub fn build(
    selection: &[ObjectInfo],
    spacing_meters: f64,
    step_per_segment: Option<usize>,
) -> Result<Vec<ObjectInfo>> {
    if spacing_meters <= 0.0 || selection.len() < 2 {
        return Err(anyhow!("Invalid spacing or selection length"));
    }

    let steps_per_segment = step_per_segment.unwrap_or(100);

    // prepare points & ghost points for centripetal catmull-rom
    let mut points = Vec::with_capacity(selection.len() + 2);
    points.insert(0, selection.first().unwrap().position().to_dvec3_metres());
    points.extend(selection.iter().map(|obj| obj.position().to_dvec3_metres()));
    points.push(selection.last().unwrap().position().to_dvec3_metres());

    let initial = spline::normalize_or_fallback(points[2] - points[1], DVec3::Y);
    let (lut, total_len) = spline::build_lut(&points, steps_per_segment, initial);

    let prototype = &selection[0];
    let proto_heading = prototype.heading().unwrap_or(Heading::NORTH);

    // generate spaced-out objects
    let num_objects = (total_len / spacing_meters).floor() as usize + 1;
    let mut output = Vec::with_capacity(num_objects);

    for i in 0..num_objects {
        // force the last object to land exactly on total_len to avoid rounding misses
        let d_target = (i as f64 * spacing_meters).min(total_len);

        let entry = spline::sample_lut(&lut, d_target);

        let max_seg_idx = points.len().saturating_sub(4);
        let mut seg_idx = entry.t.floor() as usize;
        let mut local_t = entry.t.fract();
        if seg_idx > max_seg_idx {
            seg_idx = max_seg_idx;
            local_t = 1.0;
        }

        let pos = spline::catmull_rom(&points[seg_idx..seg_idx + 4], local_t);
        let final_heading = spline::heading_from_vec2_or_fallback(entry.tangent.truncate(), proto_heading);

        let mut obj = prototype.clone();
        *obj.position_mut() = ObjectCoordinate::from_dvec3_metres(pos);
        if let Some(heading) = obj.heading_mut() {
            *heading = final_heading;
        }
        output.push(obj);
    }

    Ok(output)
}
