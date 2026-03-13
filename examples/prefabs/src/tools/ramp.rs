use anyhow::{Result, ensure};
use glam::DVec3;
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

use super::spline;

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

    let initial_tangent = spline::normalize_or_fallback(
        points[2] - points[1],
        spline::heading_to_forward(prototype.heading).extend(0.0),
    );
    let (lut, total_len) = spline::build_lut(&points, steps_per_segment, initial_tangent);

    ensure!(
        total_len > f64::EPSILON,
        "guide points produce zero horizontal path length"
    );

    let get_spline_pos = |d_target: f64| -> DVec3 {
        let entry = spline::sample_lut(&lut, d_target);
        let max_seg_idx = points.len().saturating_sub(4);
        let mut seg_idx = entry.t.floor() as usize;
        let mut local_t = entry.t.fract();
        if seg_idx > max_seg_idx {
            seg_idx = max_seg_idx;
            local_t = 1.0;
        }
        spline::catmull_rom(&points[seg_idx..seg_idx + 4], local_t)
    };

    let mut output = Vec::new();
    let mut current_distance = 0.0;
    let mut current_seam = get_spline_pos(0.0);

    let initial_next = get_spline_pos(total_len.min(0.1));
    let mut prev_heading = spline::heading_from_vec2_or_fallback(
        (initial_next - current_seam).truncate(),
        prototype.heading,
    );

    let current_bank_angle = match config.mode {
        RampMode::AlongPath => 0.0,
        RampMode::AcrossPath => config.roll_degrees,
    };

    while current_distance < total_len {
        // --- SPACING ---
        let base_step = match config.mode {
            RampMode::AlongPath => concrete_width_length_metres(prototype.length),
            RampMode::AcrossPath => concrete_width_length_metres(prototype.width),
        };

        let step_metres = match config.mode {
            RampMode::AlongPath => base_step,
            RampMode::AcrossPath => base_step * 0.50, // 50% overlap for banked surface
        };

        if step_metres <= f64::EPSILON { break; }

        // --- CALCULATE EXACT 3D CHORD ---
        let target_distance = (current_distance + step_metres).min(total_len);
        let target_pos = get_spline_pos(target_distance);
        let delta = target_pos - current_seam;

        let chord_heading = spline::heading_from_vec2_or_fallback(delta.truncate(), prev_heading);
        let actual_horizontal = delta.truncate().length();
        let slope_degrees = if actual_horizontal <= f64::EPSILON {
            0.0
        } else {
            delta.z.atan2(actual_horizontal).to_degrees()
        };

        // --- BUILD AND PLACE ---
        match config.mode {
            RampMode::AcrossPath => {
                let quarter_turn = std::f64::consts::FRAC_PI_2;
                let final_heading = if current_bank_angle < 0.0 {
                    Heading::from_radians(chord_heading.to_radians() - quarter_turn)
                } else {
                    Heading::from_radians(chord_heading.to_radians() + quarter_turn)
                };

                let fwd = spline::heading_to_forward(chord_heading);
                let actual_travel = DVec3::new(
                    fwd.x * step_metres,
                    fwd.y * step_metres,
                    delta.z, // Lock Z to spline arc
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

                let fwd = spline::heading_to_forward(chord_heading);
                let actual_travel = DVec3::new(
                    fwd.x * step_metres,
                    fwd.y * step_metres,
                    actual_rise,
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
