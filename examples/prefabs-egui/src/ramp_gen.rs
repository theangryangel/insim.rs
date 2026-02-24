use std::cmp::Ordering;

use glam::{DVec2, DVec3};
use insim_core::{
    heading::Heading,
    object::{
        concrete::{
            ConcreteHeight, ConcretePitch, ConcreteRamp, ConcreteSlab, ConcreteWidthLength,
        },
        ObjectCoordinate, ObjectInfo,
    },
};

use crate::tools::RampNode;

const XY_RAW_PER_METRE: f64 = 16.0;
const Z_RAW_PER_METRE: f64 = 4.0;
const XY_RAW_PER_Z_RAW: f64 = XY_RAW_PER_METRE / Z_RAW_PER_METRE;
const HEIGHT_STEP_XY_RAW: f64 = XY_RAW_PER_Z_RAW;

#[derive(Clone, Copy)]
struct LutEntry {
    t: f64,
    distance: f64,
    tangent: DVec3,
}

#[derive(Clone, Copy)]
struct Candidate {
    target_position: DVec3,
    travel_heading: Heading,
    travel_rise_raw: f64,
    piece: Piece,
}

#[derive(Clone, Copy)]
enum Piece {
    Slab { heading: Heading },
    Ramp { heading: Heading, height_step: u8 },
}

pub fn build(
    control_points: &[RampNode],
    template: &ObjectInfo,
    steps_per_segment: usize,
) -> Result<Vec<ObjectInfo>, String> {
    if control_points.len() < 2 {
        return Err("place at least two ramp guide points".to_owned());
    }

    let prototype = prototype_slab(template);
    let spacing_raw = concrete_width_length_xy_raw(prototype.length);
    if spacing_raw <= 0.0 {
        return Err("template slab length must be positive".to_owned());
    }

    let steps_per_segment = steps_per_segment.max(1);

    let points_source: Vec<DVec3> = control_points
        .iter()
        .copied()
        .map(node_to_raw_path)
        .collect();
    let mut points = Vec::with_capacity(points_source.len() + 2);
    points.push(points_source[0]);
    points.extend(points_source.iter().copied());
    points.push(*points_source.last().unwrap_or(&points_source[0]));

    let interpolate = |pts: &[DVec3], t_norm: f64| -> DVec3 {
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
    };

    let mut lut = Vec::with_capacity((points.len() - 3) * steps_per_segment + 1);
    let mut total_len = 0.0;
    let mut prev_pos = points[1];

    let prototype_forward = heading_to_forward(prototype.heading);
    let mut last_tangent =
        normalize_or_fallback(points[2] - points[1], prototype_forward.extend(0.0));
    lut.push(LutEntry {
        t: 0.0,
        distance: 0.0,
        tangent: last_tangent,
    });

    for i in 0..points.len() - 3 {
        let seg = &points[i..i + 4];
        for s in 1..=steps_per_segment {
            let t_local = s as f64 / steps_per_segment as f64;
            let pos = interpolate(seg, t_local);
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

    if total_len <= f64::EPSILON {
        return Err("guide points produce zero horizontal path length".to_owned());
    }

    let num_objects = (total_len / spacing_raw).ceil() as usize + 1;
    let mut candidates = Vec::with_capacity(num_objects);
    let mut fallback_heading = prototype.heading;

    for i in 0..num_objects {
        let d_target = (i as f64 * spacing_raw).min(total_len);
        let entry = sample_lut(&lut, d_target);

        let max_seg_idx = points.len() - 4;
        let mut seg_idx = entry.t.floor() as usize;
        let mut local_t = entry.t.fract();
        if seg_idx > max_seg_idx {
            seg_idx = max_seg_idx;
            local_t = 1.0;
        }

        let pos = interpolate(&points[seg_idx..seg_idx + 4], local_t);
        let tangent = normalize_or_fallback(
            entry.tangent,
            heading_to_forward(fallback_heading).extend(0.0),
        );
        let path_heading = heading_from_vec2_or_fallback(tangent.truncate(), fallback_heading);

        let horizontal = tangent.truncate().length();
        let slope_degrees = if horizontal <= f64::EPSILON {
            0.0
        } else {
            tangent.z.atan2(horizontal).to_degrees()
        };

        let rise_raw = slope_degrees.abs().to_radians().tan() * spacing_raw;
        let height_step = quantize_height_step(rise_raw);

        let piece = if height_step == 0 {
            Piece::Slab {
                heading: path_heading,
            }
        } else {
            let heading = if slope_degrees < 0.0 {
                path_heading.opposite()
            } else {
                path_heading
            };
            Piece::Ramp {
                heading,
                height_step,
            }
        };

        fallback_heading = match piece {
            Piece::Slab { heading } => heading,
            Piece::Ramp { heading, .. } => heading,
        };

        let travel_rise_raw = match piece {
            Piece::Ramp { height_step, .. } => {
                let magnitude = height_raw_from_step(height_step);
                if slope_degrees < 0.0 {
                    -magnitude
                } else {
                    magnitude
                }
            },
            Piece::Slab { .. } => 0.0,
        };

        candidates.push(Candidate {
            target_position: pos,
            travel_heading: path_heading,
            travel_rise_raw,
            piece,
        });
    }

    let mut centres = Vec::with_capacity(candidates.len());
    for (idx, candidate) in candidates.iter().enumerate() {
        let center = if idx == 0 {
            candidate.target_position
        } else {
            let prev_center = centres[idx - 1];
            let prev_forward = travel_half_offset(
                candidates[idx - 1].travel_heading,
                candidates[idx - 1].travel_rise_raw,
                spacing_raw,
            );
            let this_forward = travel_half_offset(
                candidate.travel_heading,
                candidate.travel_rise_raw,
                spacing_raw,
            );
            let seam = prev_center + prev_forward;
            seam + this_forward
        };
        centres.push(center);
    }

    let mut output = Vec::with_capacity(candidates.len());
    for (idx, candidate) in candidates.iter().enumerate() {
        match candidate.piece {
            Piece::Slab { heading } => {
                let mut slab = prototype.clone();
                slab.xyz = object_coordinate_from_raw_path(centres[idx]);
                slab.heading = heading;
                slab.pitch = ConcretePitch::Deg0;
                output.push(ObjectInfo::ConcreteSlab(slab));
            },
            Piece::Ramp {
                heading,
                height_step,
            } => {
                output.push(ObjectInfo::ConcreteRamp(ConcreteRamp {
                    xyz: object_coordinate_from_raw_path(centres[idx]),
                    width: prototype.width,
                    length: prototype.length,
                    height: height_from_step(height_step),
                    heading,
                }));
            },
        }
    }

    Ok(output)
}

fn node_to_raw_path(node: RampNode) -> DVec3 {
    DVec3::new(
        f64::from(node.x),
        f64::from(node.y),
        f64::from(node.z) * XY_RAW_PER_Z_RAW,
    )
}

fn object_coordinate_from_raw_path(pos: DVec3) -> ObjectCoordinate {
    let x = pos.x.round().clamp(i16::MIN as f64, i16::MAX as f64) as i16;
    let y = pos.y.round().clamp(i16::MIN as f64, i16::MAX as f64) as i16;
    let z = (pos.z / XY_RAW_PER_Z_RAW)
        .round()
        .clamp(0.0, f64::from(u8::MAX)) as u8;
    ObjectCoordinate::new(x, y, z)
}

fn sample_lut(lut: &[LutEntry], target_distance: f64) -> LutEntry {
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
                *lut.last().unwrap_or(&lut[0])
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
        },
    }
}

fn heading_from_vec2_or_fallback(vector: DVec2, fallback: Heading) -> Heading {
    if vector.length_squared() <= f64::EPSILON {
        return fallback;
    }

    let tangent = vector.normalize();
    Heading::from_radians((-tangent.x).atan2(tangent.y))
}

fn heading_to_forward(heading: Heading) -> DVec2 {
    let radians = heading.to_radians();
    DVec2::new(-radians.sin(), radians.cos())
}

fn normalize_or_fallback(vector: DVec3, fallback: DVec3) -> DVec3 {
    if vector.length_squared() > f64::EPSILON {
        vector.normalize()
    } else if fallback.length_squared() > f64::EPSILON {
        fallback.normalize()
    } else {
        DVec3::Y
    }
}

fn travel_half_offset(heading: Heading, rise_raw: f64, spacing_raw: f64) -> DVec3 {
    let forward = heading_to_forward(heading);
    let half = spacing_raw * 0.5;
    DVec3::new(forward.x * half, forward.y * half, rise_raw * 0.5)
}

fn prototype_slab(template: &ObjectInfo) -> ConcreteSlab {
    match template {
        ObjectInfo::ConcreteSlab(slab) => slab.clone(),
        ObjectInfo::ConcreteRamp(ramp) => ConcreteSlab {
            xyz: ObjectCoordinate::default(),
            width: ramp.width,
            length: ramp.length,
            pitch: ConcretePitch::Deg0,
            heading: ramp.heading,
        },
        _ => default_slab(),
    }
}

fn default_slab() -> ConcreteSlab {
    ConcreteSlab {
        xyz: ObjectCoordinate::default(),
        width: ConcreteWidthLength::Four,
        length: ConcreteWidthLength::Four,
        pitch: ConcretePitch::Deg0,
        heading: Heading::NORTH,
    }
}

fn concrete_width_length_xy_raw(length: ConcreteWidthLength) -> f64 {
    match length {
        ConcreteWidthLength::Two => 2.0 * XY_RAW_PER_METRE,
        ConcreteWidthLength::Four => 4.0 * XY_RAW_PER_METRE,
        ConcreteWidthLength::Eight => 8.0 * XY_RAW_PER_METRE,
        ConcreteWidthLength::Sixteen => 16.0 * XY_RAW_PER_METRE,
        _ => 4.0 * XY_RAW_PER_METRE,
    }
}

fn quantize_height_step(rise_raw: f64) -> u8 {
    let clamped = rise_raw.abs().clamp(0.0, 4.0 * XY_RAW_PER_METRE);
    let step = (clamped / HEIGHT_STEP_XY_RAW).round() as i32;
    step.clamp(0, 16) as u8
}

fn height_raw_from_step(step: u8) -> f64 {
    f64::from(step.min(16)) * HEIGHT_STEP_XY_RAW
}

fn height_from_step(step: u8) -> ConcreteHeight {
    let wire = step.saturating_sub(1).min(15);
    ConcreteHeight::try_from(wire).unwrap_or(ConcreteHeight::M4_00)
}
