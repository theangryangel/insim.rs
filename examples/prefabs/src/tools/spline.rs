use std::cmp::Ordering;

use glam::{DVec2, DVec3};
use insim::core::heading::Heading;

#[derive(Debug, Clone, Copy)]
pub struct LutEntry {
    pub t: f64,
    pub distance: f64,
    pub tangent: DVec3,
}

/// Centripetal Catmull-Rom interpolation over 4 consecutive points.
/// `t_norm` is in [0, 1] and maps to the segment between pts[1] and pts[2].
pub fn catmull_rom(pts: &[DVec3], t_norm: f64) -> DVec3 {
    let alpha = 0.5;
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
}

/// Build an arc-length LUT from a ghost-padded points slice.
/// Returns `(lut, total_horizontal_length)`.
/// `initial_tangent` seeds the first LUT entry and must already be normalised.
pub fn build_lut(
    points: &[DVec3],
    steps_per_segment: usize,
    initial_tangent: DVec3,
) -> (Vec<LutEntry>, f64) {
    let mut lut = Vec::with_capacity((points.len() - 3) * steps_per_segment + 1);
    let mut total_len = 0.0f64;
    let mut prev_pos = points[1];
    let mut last_tangent = initial_tangent;

    lut.push(LutEntry {
        t: 0.0,
        distance: 0.0,
        tangent: last_tangent,
    });

    for i in 0..points.len() - 3 {
        let seg = &points[i..i + 4];
        for s in 1..=steps_per_segment {
            let t_local = s as f64 / steps_per_segment as f64;
            let pos = catmull_rom(seg, t_local);
            let delta = pos - prev_pos;
            let dist_xy = delta.truncate().length();

            if dist_xy > f64::EPSILON {
                total_len += dist_xy;
                last_tangent = normalize_or_fallback(delta, last_tangent);
                lut.push(LutEntry {
                    t: i as f64 + t_local,
                    distance: total_len,
                    tangent: last_tangent,
                });
            }
            prev_pos = pos;
        }
    }

    (lut, total_len)
}

/// Sample the LUT at a target arc-length distance, interpolating between entries.
pub fn sample_lut(lut: &[LutEntry], target_distance: f64) -> LutEntry {
    match lut.binary_search_by(|entry| {
        entry
            .distance
            .partial_cmp(&target_distance)
            .unwrap_or(Ordering::Less)
    }) {
        Ok(idx) => lut[idx],
        Err(idx) => {
            if idx == 0 {
                lut[0]
            } else if idx >= lut.len() {
                *lut.last().unwrap()
            } else {
                let e0 = lut[idx - 1];
                let e1 = lut[idx];
                let span = e1.distance - e0.distance;
                if span <= f64::EPSILON {
                    e0
                } else {
                    let factor = (target_distance - e0.distance) / span;
                    LutEntry {
                        t: e0.t + (e1.t - e0.t) * factor,
                        distance: target_distance,
                        tangent: normalize_or_fallback(
                            e0.tangent.lerp(e1.tangent, factor),
                            e0.tangent,
                        ),
                    }
                }
            }
        }
    }
}

pub fn heading_to_forward(heading: Heading) -> DVec2 {
    let radians = heading.to_radians();
    DVec2::new(-radians.sin(), radians.cos())
}

pub fn heading_from_vec2_or_fallback(vector: DVec2, fallback: Heading) -> Heading {
    if vector.length_squared() <= f64::EPSILON {
        return fallback;
    }
    let tangent = vector.normalize();
    Heading::from_radians((-tangent.x).atan2(tangent.y))
}

pub fn normalize_or_fallback(vector: DVec3, fallback: DVec3) -> DVec3 {
    if vector.length_squared() > f64::EPSILON {
        vector.normalize()
    } else if fallback.length_squared() > f64::EPSILON {
        fallback.normalize()
    } else {
        DVec3::Y
    }
}
