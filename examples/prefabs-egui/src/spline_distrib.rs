use std::cmp::Ordering;

use glam::DVec2;
use insim_core::{heading::Heading, object::ObjectInfo};

const DEFAULT_STEPS_PER_SEGMENT: usize = 100;

#[derive(Debug, Clone, Copy)]
struct LutEntry {
    t: f64,
    distance: f64,
    tangent: DVec2,
}

#[derive(Debug, Clone)]
struct SplineLut {
    points: Vec<DVec2>,
    lut: Vec<LutEntry>,
    total_len: f64,
    curve_points: Vec<[f64; 2]>,
}

#[derive(Debug, Clone, Copy)]
pub struct SplineSampleRaw {
    pub pos: [i16; 2],
    pub tangent: [f64; 2],
}

pub fn preview_curve_points(control_points: &[[i16; 2]]) -> Vec<[f64; 2]> {
    build_lut(control_points, DEFAULT_STEPS_PER_SEGMENT)
        .map(|lut| lut.curve_points)
        .unwrap_or_default()
}

pub fn sample_spaced_raw(
    control_points: &[[i16; 2]],
    spacing_units: i32,
    steps_per_segment: Option<usize>,
) -> Result<Vec<SplineSampleRaw>, String> {
    if spacing_units <= 0 {
        return Err("Invalid spacing or control point length".to_owned());
    }

    let lut = build_lut(
        control_points,
        steps_per_segment.unwrap_or(DEFAULT_STEPS_PER_SEGMENT),
    )?;

    if lut.lut.is_empty() || lut.total_len <= f64::EPSILON {
        return Err("Spline length is zero".to_owned());
    }

    let spacing = f64::from(spacing_units);
    let num_objects = (lut.total_len / spacing).floor() as usize + 1;
    let mut output = Vec::with_capacity(num_objects);

    for idx in 0..num_objects {
        let d_target = (idx as f64 * spacing).min(lut.total_len);

        let entry = match lut.lut.binary_search_by(|entry| {
            entry
                .distance
                .partial_cmp(&d_target)
                .unwrap_or(Ordering::Less)
        }) {
            Ok(entry_idx) => lut.lut[entry_idx],
            Err(entry_idx) => {
                if entry_idx == 0 {
                    lut.lut[0]
                } else if entry_idx >= lut.lut.len() {
                    *lut.lut.last().unwrap_or(&lut.lut[0])
                } else {
                    let e0 = lut.lut[entry_idx - 1];
                    let e1 = lut.lut[entry_idx];
                    let distance_span = e1.distance - e0.distance;
                    let factor = if distance_span.abs() <= f64::EPSILON {
                        0.0
                    } else {
                        (d_target - e0.distance) / distance_span
                    };
                    LutEntry {
                        t: e0.t + (e1.t - e0.t) * factor,
                        distance: d_target,
                        tangent: normalize_or(e0.tangent.lerp(e1.tangent, factor), e0.tangent),
                    }
                }
            },
        };

        let mut seg_idx = entry.t.floor() as usize;
        let mut local_t = entry.t.fract();
        if seg_idx + 4 > lut.points.len() {
            seg_idx = lut.points.len().saturating_sub(4);
            local_t = 1.0;
        }
        let pos = interpolate(&lut.points[seg_idx..seg_idx + 4], local_t);

        output.push(SplineSampleRaw {
            pos: [
                pos.x.round().clamp(i16::MIN as f64, i16::MAX as f64) as i16,
                pos.y.round().clamp(i16::MIN as f64, i16::MAX as f64) as i16,
            ],
            tangent: [entry.tangent.x, entry.tangent.y],
        });
    }

    Ok(output)
}

pub fn build(
    control_points: &[[i16; 2]],
    template: &ObjectInfo,
    spacing_units: i32,
    steps_per_segment: Option<usize>,
) -> Result<Vec<ObjectInfo>, String> {
    let samples = sample_spaced_raw(control_points, spacing_units, steps_per_segment)?;
    if samples.is_empty() {
        return Err("Spline apply produced no points".to_owned());
    }

    let first_tangent = DVec2::new(samples[0].tangent[0], samples[0].tangent[1]);
    let heading_offset = template
        .heading()
        .map(|heading| heading.to_radians() - heading_from_vec2(first_tangent).to_radians())
        .unwrap_or(0.0);

    let mut objects = Vec::with_capacity(samples.len());
    for sample in samples {
        let mut object = template.clone();
        let position = object.position_mut();
        position.x = sample.pos[0];
        position.y = sample.pos[1];

        if let Some(heading) = object.heading_mut() {
            let tangent = DVec2::new(sample.tangent[0], sample.tangent[1]);
            *heading =
                Heading::from_radians(heading_from_vec2(tangent).to_radians() + heading_offset);
        }

        objects.push(object);
    }

    Ok(objects)
}

fn build_lut(control_points: &[[i16; 2]], steps_per_segment: usize) -> Result<SplineLut, String> {
    if control_points.len() < 2 || steps_per_segment == 0 {
        return Err("Invalid spacing or control point length".to_owned());
    }

    let mut points = Vec::with_capacity(control_points.len() + 2);
    let first = DVec2::new(
        f64::from(control_points[0][0]),
        f64::from(control_points[0][1]),
    );
    points.push(first);
    points.extend(
        control_points
            .iter()
            .map(|point| DVec2::new(f64::from(point[0]), f64::from(point[1]))),
    );
    let last = DVec2::new(
        f64::from(control_points[control_points.len() - 1][0]),
        f64::from(control_points[control_points.len() - 1][1]),
    );
    points.push(last);

    let mut lut = Vec::with_capacity((points.len() - 3) * steps_per_segment + 1);
    let mut curve_points = Vec::with_capacity((points.len() - 3) * steps_per_segment + 1);
    let mut total_len = 0.0;
    let mut prev_pos = points[1];
    let initial_tangent = normalize_or(points[2] - points[1], DVec2::Y);

    lut.push(LutEntry {
        t: 0.0,
        distance: 0.0,
        tangent: initial_tangent,
    });
    curve_points.push([prev_pos.x, prev_pos.y]);

    for i in 0..points.len() - 3 {
        let seg = &points[i..i + 4];
        for s in 1..=steps_per_segment {
            let t_local = s as f64 / steps_per_segment as f64;
            let pos = interpolate(seg, t_local);

            let delta = pos - prev_pos;
            let dist = delta.length();

            if dist > f64::EPSILON {
                total_len += dist;
                lut.push(LutEntry {
                    t: i as f64 + t_local,
                    distance: total_len,
                    tangent: normalize_or(delta, initial_tangent),
                });
                curve_points.push([pos.x, pos.y]);
            }

            prev_pos = pos;
        }
    }

    Ok(SplineLut {
        points,
        lut,
        total_len,
        curve_points,
    })
}

fn interpolate(points: &[DVec2], t_norm: f64) -> DVec2 {
    let alpha = 0.5;
    let dt0 = points[0].distance(points[1]).powf(alpha);
    let dt1 = points[1].distance(points[2]).powf(alpha);
    let dt2 = points[2].distance(points[3]).powf(alpha);

    let t1 = dt0;
    let t2 = t1 + dt1;
    let t3 = t2 + dt2;

    if dt1 < f64::EPSILON {
        return points[1];
    }

    let target_t = t1 + t_norm * (t2 - t1);

    let lerp = |a: DVec2, b: DVec2, ta: f64, tb: f64| {
        if (tb - ta).abs() < f64::EPSILON {
            a
        } else {
            (tb - target_t) / (tb - ta) * a + (target_t - ta) / (tb - ta) * b
        }
    };

    let a1 = lerp(points[0], points[1], 0.0, t1);
    let a2 = lerp(points[1], points[2], t1, t2);
    let a3 = lerp(points[2], points[3], t2, t3);
    let b1 = lerp(a1, a2, 0.0, t2);
    let b2 = lerp(a2, a3, t1, t3);
    lerp(b1, b2, t1, t2)
}

fn heading_from_vec2(vector: DVec2) -> Heading {
    Heading::from_radians((-vector.x).atan2(vector.y))
}

fn normalize_or(vector: DVec2, fallback: DVec2) -> DVec2 {
    if vector.length_squared() <= f64::EPSILON {
        fallback
    } else {
        vector.normalize()
    }
}

#[cfg(test)]
mod tests {
    use super::{preview_curve_points, sample_spaced_raw};

    #[test]
    fn samples_include_first_and_last_point() {
        let control_points = vec![[0, 0], [160, 0]];
        let samples = sample_spaced_raw(&control_points, 80, Some(100)).expect("samples");

        assert_eq!(samples.len(), 3);
        assert_eq!(samples[0].pos, [0, 0]);
        assert_eq!(samples[2].pos, [160, 0]);
    }

    #[test]
    fn preview_curve_not_empty_for_two_points() {
        let curve = preview_curve_points(&[[0, 0], [160, 0]]);
        assert!(!curve.is_empty());
    }
}
