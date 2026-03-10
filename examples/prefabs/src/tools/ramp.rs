use std::cmp::Ordering;

use anyhow::{Result, ensure};
use glam::{DVec2, DVec3};
use insim::{
    core::{
        heading::Heading,
        object::{
            ObjectCoordinate,
            concrete::{
                ConcreteHeight, ConcretePitch, ConcreteRamp, ConcreteSlab, ConcreteWidthLength,
            },
        },
    },
    insim::ObjectInfo,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RampMode {
    #[default]
    AlongPath,  // Classic track surface: uses wedges and flat slabs
    AcrossPath, // Banked surface: sideways slabs with dynamic easing and overlapping
}

impl RampMode {
    pub fn toggled(self) -> Self {
        match self {
            Self::AlongPath => Self::AcrossPath,
            Self::AcrossPath => Self::AlongPath,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct BuildConfig {
    pub mode: RampMode,
    pub roll_degrees: f64,
    pub steps_per_segment: Option<usize>,
}

impl Default for BuildConfig {
    fn default() -> Self {
        Self {
            mode: RampMode::AlongPath,
            roll_degrees: 18.0,
            steps_per_segment: None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct LutEntry {
    t: f64,
    distance: f64,
    tangent: DVec3,
}

pub fn build(selection: &[ObjectInfo], config: BuildConfig) -> Result<Vec<ObjectInfo>> {
    ensure!(
        selection.len() >= 2,
        "select at least two guide objects to build a ramp"
    );
    ensure!(
        config.roll_degrees.is_finite(),
        "roll must be a finite number"
    );

    let prototype = prototype_slab(selection);
    let steps_per_segment = config.steps_per_segment.unwrap_or(100).max(1);

    let first = selection
        .first()
        .map(|obj| obj.position().to_dvec3_metres())
        .unwrap();
    let last = selection
        .last()
        .map(|obj| obj.position().to_dvec3_metres())
        .unwrap();

    let mut points = Vec::with_capacity(selection.len() + 2);
    points.push(first);
    points.extend(selection.iter().map(|obj| obj.position().to_dvec3_metres()));
    points.push(last);

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
    let mut last_tangent = normalize_or_fallback(points[2] - points[1], prototype_forward.extend(0.0));
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

    ensure!(
        total_len > f64::EPSILON,
        "guide points produce zero horizontal path length"
    );

    let get_spline_pos = |d_target: f64| -> DVec3 {
        let entry = sample_lut(&lut, d_target);
        let max_seg_idx = points.len().saturating_sub(4);
        let mut seg_idx = entry.t.floor() as usize;
        let mut local_t = entry.t.fract();
        if seg_idx > max_seg_idx {
            seg_idx = max_seg_idx;
            local_t = 1.0;
        }
        interpolate(&points[seg_idx..seg_idx + 4], local_t)
    };

    let mut output = Vec::new();
    let mut current_distance = 0.0;
    let mut current_seam = get_spline_pos(0.0);
    
    let initial_next = get_spline_pos(total_len.min(0.1));
    let mut prev_heading = heading_from_vec2_or_fallback((initial_next - current_seam).truncate(), prototype.heading);

    let mut current_bank_angle = 0.0;
    let max_roll_change_per_block = 6.0;

    while current_distance < total_len {
        let target_bank = match config.mode {
            RampMode::AlongPath => 0.0,
            RampMode::AcrossPath => config.roll_degrees,
        };

        // --- EASE THE TRANSITION ---
        let bank_diff = target_bank - current_bank_angle;
        let is_transitioning = bank_diff.abs() > f64::EPSILON;

        // If the block has ANY roll, it MUST be built sideways
        let active_orientation = if current_bank_angle.abs() > f64::EPSILON || target_bank.abs() > f64::EPSILON {
            RampMode::AcrossPath
        } else {
            RampMode::AlongPath
        };

        // --- DYNAMIC SPACING (Micro-stepping for smoothness) ---
        let base_step = match active_orientation {
            RampMode::AlongPath => concrete_width_length_metres(prototype.length),
            RampMode::AcrossPath => concrete_width_length_metres(prototype.width),
        };

        let step_metres = if is_transitioning {
            base_step * 0.25 // 75% overlap while twisting
        } else if current_bank_angle.abs() > f64::EPSILON {
            base_step * 0.50 // 50% overlap during sustained bank
        } else {
            base_step        // 0% overlap on flat straights
        };

        if step_metres <= f64::EPSILON { break; }

        if is_transitioning {
            let change = bank_diff.signum() * bank_diff.abs().min(max_roll_change_per_block);
            current_bank_angle += change;
        }

        // --- CALCULATE EXACT 3D CHORD ---
        let target_distance = (current_distance + step_metres).min(total_len);
        let target_pos = get_spline_pos(target_distance);
        let delta = target_pos - current_seam;
        
        let chord_heading = heading_from_vec2_or_fallback(delta.truncate(), prev_heading);
        let actual_horizontal = delta.truncate().length();
        let slope_degrees = if actual_horizontal <= f64::EPSILON {
            0.0
        } else {
            delta.z.atan2(actual_horizontal).to_degrees()
        };

        // --- BUILD AND PLACE ---
        match active_orientation {
            RampMode::AcrossPath => {
                let quarter_turn = std::f64::consts::FRAC_PI_2;
                let final_heading = if current_bank_angle < 0.0 {
                    Heading::from_radians(chord_heading.to_radians() - quarter_turn)
                } else {
                    Heading::from_radians(chord_heading.to_radians() + quarter_turn)
                };

                let actual_travel = DVec3::new(
                    heading_to_forward(chord_heading).x * step_metres,
                    heading_to_forward(chord_heading).y * step_metres,
                    delta.z // Lock Z to spline arc
                );

                let center = current_seam + actual_travel * 0.5;
                
                let mut slab = prototype.clone();
                slab.xyz = ObjectCoordinate::from_dvec3_metres(center);
                slab.heading = final_heading;
                slab.pitch = pitch_from_step(quantize_pitch_step(current_bank_angle.abs()));
                output.push(ObjectInfo::ConcreteSlab(slab));

                current_seam += actual_travel;
            },
            RampMode::AlongPath => {
                let rise_metres = slope_degrees.abs().to_radians().tan() * step_metres;
                let height_step = quantize_height_step(rise_metres);
                let magnitude = height_metres_from_step(height_step);
                let actual_rise = if slope_degrees < 0.0 { -magnitude } else { magnitude };

                let actual_travel = DVec3::new(
                    heading_to_forward(chord_heading).x * step_metres,
                    heading_to_forward(chord_heading).y * step_metres,
                    actual_rise
                );

                let center = current_seam + actual_travel * 0.5;

                if height_step == 0 {
                    let mut slab = prototype.clone();
                    slab.xyz = ObjectCoordinate::from_dvec3_metres(center);
                    slab.heading = chord_heading;
                    slab.pitch = ConcretePitch::Deg0;
                    output.push(ObjectInfo::ConcreteSlab(slab));
                } else {
                    let block_heading = if slope_degrees < 0.0 { chord_heading.opposite() } else { chord_heading };
                    output.push(ObjectInfo::ConcreteRamp(ConcreteRamp {
                        xyz: ObjectCoordinate::from_dvec3_metres(center),
                        width: prototype.width,
                        length: prototype.length,
                        height: height_from_step(height_step),
                        heading: block_heading,
                    }));
                }

                current_seam += actual_travel;
            }
        }

        prev_heading = chord_heading;
        current_distance += step_metres;
    }

    Ok(output)
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

fn prototype_slab(selection: &[ObjectInfo]) -> ConcreteSlab {
    if let Some(slab) = selection.iter().find_map(|obj| match obj {
        ObjectInfo::ConcreteSlab(slab) => Some(slab.clone()),
        _ => None,
    }) {
        return slab;
    }

    if let Some(ramp) = selection.iter().find_map(|obj| match obj {
        ObjectInfo::ConcreteRamp(ramp) => Some(ramp.clone()),
        _ => None,
    }) {
        return ConcreteSlab {
            xyz: ObjectCoordinate::default(),
            width: ramp.width,
            length: ramp.length,
            pitch: ConcretePitch::Deg0,
            heading: ramp.heading,
        };
    }

    default_slab()
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

fn concrete_width_length_metres(length: ConcreteWidthLength) -> f64 {
    match length {
        ConcreteWidthLength::Two => 2.0,
        ConcreteWidthLength::Four => 4.0,
        ConcreteWidthLength::Eight => 8.0,
        ConcreteWidthLength::Sixteen => 16.0,
        _ => 4.0,
    }
}

fn quantize_pitch_step(degrees: f64) -> u8 {
    let clamped = degrees.abs().clamp(0.0, 90.0);
    let step = (clamped / 6.0).round() as i32;
    step.clamp(0, 15) as u8
}

fn pitch_from_step(step: u8) -> ConcretePitch {
    ConcretePitch::try_from(step.min(15)).unwrap_or(ConcretePitch::Deg90)
}

fn quantize_height_step(metres: f64) -> u8 {
    let clamped = metres.abs().clamp(0.0, 4.0);
    let step = (clamped / 0.25).round() as i32;
    step.clamp(0, 16) as u8
}

fn height_metres_from_step(step: u8) -> f64 {
    f64::from(step.min(16)) * 0.25
}

fn height_from_step(step: u8) -> ConcreteHeight {
    let wire = step.saturating_sub(1).min(15);
    ConcreteHeight::try_from(wire).unwrap_or(ConcreteHeight::M4_00)
}
