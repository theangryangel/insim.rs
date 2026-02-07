use std::cmp::Ordering;

use anyhow::{Result, anyhow};
use glam::{DVec2, DVec3};
use insim::{
    core::{heading::Heading, object::ObjectCoordinate},
    insim::ObjectInfo,
};

#[derive(Debug, Clone, Copy)]
struct LutEntry {
    t: f64,
    distance: f64,
    tangent: DVec2,
}

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

    // spline interpolation helper (centripetal catmull-rom)
    let interpolate = |pts: &[DVec3], t_norm: f64| -> DVec3 {
        let alpha = 0.5; // centripetal
        let dt0 = pts[0].distance(pts[1]).powf(alpha);
        let dt1 = pts[1].distance(pts[2]).powf(alpha);
        let dt2 = pts[2].distance(pts[3]).powf(alpha);

        let t1 = dt0;
        let t2 = t1 + dt1;
        let t3 = t2 + dt2;

        if dt1 < f64::EPSILON {
            return pts[1];
        }
        let target_t = t1 + t_norm * (t2 - t1);

        let lerp = |a: DVec3, b: DVec3, ta: f64, tb: f64| {
            if (tb - ta).abs() < f64::EPSILON {
                a
            } else {
                (tb - target_t) / (tb - ta) * a + (target_t - ta) / (tb - ta) * b
            }
        };

        let a1 = lerp(pts[0], pts[1], 0.0, t1);
        let a2 = lerp(pts[1], pts[2], t1, t2);
        let a3 = lerp(pts[2], pts[3], t2, t3);
        let b1 = lerp(a1, a2, 0.0, t2);
        let b2 = lerp(a2, a3, t1, t3);
        lerp(b1, b2, t1, t2)
    };

    // build look-up table
    let mut lut = Vec::with_capacity((points.len() - 3) * steps_per_segment + 1);
    let mut total_len = 0.0;
    let mut prev_pos = points[1];

    // initial entry for the very first point
    lut.push(LutEntry {
        t: 0.0,
        distance: 0.0,
        tangent: (points[2].truncate() - points[1].truncate()).normalize(),
    });

    for i in 0..points.len() - 3 {
        let seg = &points[i..i + 4];
        for s in 1..=steps_per_segment {
            let t_local = s as f64 / steps_per_segment as f64;
            let pos = interpolate(seg, t_local);

            let delta = pos.truncate() - prev_pos.truncate();
            let dist = delta.length();

            if dist > f64::EPSILON {
                total_len += dist;
                lut.push(LutEntry {
                    t: i as f64 + t_local,
                    distance: total_len,
                    tangent: delta.normalize(),
                });
            }
            prev_pos = pos;
        }
    }

    // calculate prototype offset (align to first object's placement)
    let prototype = &selection[0];
    let first_tangent = lut[0].tangent;
    let heading_from_vec2 = |v: DVec2| Heading::from_radians((-v.x).atan2(v.y));

    let heading_offset = prototype
        .heading()
        .map(|h| h.to_radians() - heading_from_vec2(first_tangent).to_radians())
        .unwrap_or(0.0);

    // generate spaced-out objects
    let num_objects = (total_len / spacing_meters).floor() as usize + 1;
    let mut output = Vec::with_capacity(num_objects);

    for i in 0..num_objects {
        // force the last object to land exactly on total_len to avoid rounding misses
        let d_target = (i as f64 * spacing_meters).min(total_len);

        // find and interpolate lut entries
        let entry = match lut
            .binary_search_by(|e| e.distance.partial_cmp(&d_target).unwrap_or(Ordering::Less))
        {
            Ok(idx) => lut[idx],
            Err(idx) => {
                if idx == 0 {
                    lut[0]
                } else if idx >= lut.len() {
                    *lut.last().unwrap()
                } else {
                    let e0 = lut[idx - 1];
                    let e1 = lut[idx];
                    let factor = (d_target - e0.distance) / (e1.distance - e0.distance);
                    LutEntry {
                        t: e0.t + (e1.t - e0.t) * factor,
                        distance: d_target,
                        tangent: e0.tangent.lerp(e1.tangent, factor).normalize(),
                    }
                }
            },
        };

        let seg_idx = entry.t.floor() as usize;
        let local_t = entry.t.fract();
        let pos = interpolate(&points[seg_idx..seg_idx + 4], local_t);
        let final_heading =
            Heading::from_radians(heading_from_vec2(entry.tangent).to_radians() + heading_offset);

        let mut obj = prototype.clone();
        *obj.position_mut() = ObjectCoordinate::from_dvec3_metres(pos);
        let _ = obj.set_heading(final_heading);
        output.push(obj);
    }

    Ok(output)
}
