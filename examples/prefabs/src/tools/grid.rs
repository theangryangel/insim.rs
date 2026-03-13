use anyhow::{Result, ensure};
use glam::{DVec2, DVec3};
use insim::{
    core::{
        heading::Heading,
        object::{
            ObjectCoordinate,
            pit::PitStopBox,
            pit_start_point::PitStartPoint,
            start_position::StartPosition,
        },
    },
    insim::ObjectInfo,
};

use super::spline;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GridMode {
    #[default]
    StartGrid, // → StartPosition
    Pit,       // → PitStartPoint
    PitBox,    // → PitStopBox
}

impl GridMode {
    pub fn cycled(self) -> Self {
        match self {
            Self::StartGrid => Self::Pit,
            Self::Pit => Self::PitBox,
            Self::PitBox => Self::StartGrid,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct BuildConfig {
    pub mode: GridMode,
    // flat grid (1 guide object)
    pub width: usize,
    pub rows: usize,
    pub col_spacing: f64,
    pub row_spacing: f64,
    // spline (2+ guide objects)
    pub lateral_offset: f64,
}

impl Default for BuildConfig {
    fn default() -> Self {
        Self {
            mode: GridMode::default(),
            width: 2,
            rows: 24,
            col_spacing: 4.0,
            row_spacing: 8.0,
            lateral_offset: 3.0,
        }
    }
}

pub fn build(selection: &[ObjectInfo], config: BuildConfig) -> Result<Vec<ObjectInfo>> {
    ensure!(
        !selection.is_empty(),
        "select at least one guide object to build a grid"
    );

    if selection.len() == 1 {
        build_flat(selection, config)
    } else {
        build_spline(selection, config)
    }
}

fn mode_max_index(mode: GridMode) -> usize {
    match mode {
        GridMode::StartGrid | GridMode::Pit => 47,
        GridMode::PitBox => 15,
    }
}

fn make_object(
    mode: GridMode,
    pos: DVec3,
    heading: Heading,
    floating: bool,
    index: u8,
) -> ObjectInfo {
    let heading = heading.opposite();
    match mode {
        GridMode::StartGrid => ObjectInfo::StartPosition(StartPosition {
            xyz: ObjectCoordinate::from_dvec3_metres(pos),
            heading,
            index,
            floating,
        }),
        GridMode::Pit => ObjectInfo::PitStartPoint(PitStartPoint {
            xyz: ObjectCoordinate::from_dvec3_metres(pos),
            heading,
            index,
            floating,
        }),
        GridMode::PitBox => ObjectInfo::PitStopBox(PitStopBox {
            xyz: ObjectCoordinate::from_dvec3_metres(pos),
            heading,
            colour: 0,
            mapping: index,
            floating,
        }),
    }
}

fn build_flat(selection: &[ObjectInfo], config: BuildConfig) -> Result<Vec<ObjectInfo>> {
    ensure!(config.width > 0, "width must be at least 1");
    ensure!(config.rows > 0, "rows must be at least 1");
    ensure!(
        config.col_spacing.is_finite() && config.col_spacing > 0.0,
        "col_spacing must be a positive number"
    );
    ensure!(
        config.row_spacing.is_finite() && config.row_spacing > 0.0,
        "row_spacing must be a positive number"
    );

    let prototype = selection.first().unwrap();
    let origin = prototype.position().to_dvec3_metres();
    let heading = prototype.heading().unwrap_or(Heading::NORTH);
    let floating = prototype.floating().unwrap_or(false);
    let z = origin.z;

    let forward = spline::heading_to_forward(heading);
    let right = DVec2::new(forward.y, -forward.x); // 90° clockwise

    let max_idx = mode_max_index(config.mode);
    let mut output = Vec::new();
    let mut seq = 0usize;

    for row in 0..config.rows {
        for col in 0..config.width {
            let col_offset =
                (col as f64 - (config.width as f64 - 1.0) / 2.0) * config.col_spacing;
            let row_offset = row as f64 * config.row_spacing;

            let pos = DVec3::new(
                origin.x + forward.x * row_offset + right.x * col_offset,
                origin.y + forward.y * row_offset + right.y * col_offset,
                z,
            );

            let clamped = seq.min(max_idx) as u8;
            output.push(make_object(config.mode, pos, heading, floating, clamped));
            seq += 1;
        }
    }

    Ok(output)
}

fn build_spline(selection: &[ObjectInfo], config: BuildConfig) -> Result<Vec<ObjectInfo>> {
    ensure!(
        config.row_spacing.is_finite() && config.row_spacing > 0.0,
        "row_spacing must be a positive number"
    );
    ensure!(
        config.lateral_offset.is_finite(),
        "lateral_offset must be a finite number"
    );

    let prototype = selection.first().unwrap();
    let proto_heading = prototype.heading().unwrap_or(Heading::NORTH);
    let floating = prototype.floating().unwrap_or(false);

    let steps_per_segment = 100usize;

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
        spline::heading_to_forward(proto_heading).extend(0.0),
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

    let max_idx = mode_max_index(config.mode);
    let mut output = Vec::new();
    let mut current_distance = 0.0f64;
    let mut seq = 0usize;

    while current_distance <= total_len + f64::EPSILON {
        let entry = spline::sample_lut(&lut, current_distance);
        let pos = get_spline_pos(current_distance);

        let heading = spline::heading_from_vec2_or_fallback(entry.tangent.truncate(), proto_heading);

        let fwd = spline::heading_to_forward(heading);
        let right = DVec2::new(fwd.y, -fwd.x); // 90° clockwise

        let lateral = if seq % 2 == 0 {
            config.lateral_offset
        } else {
            -config.lateral_offset
        };

        let offset_pos = DVec3::new(
            pos.x + right.x * lateral,
            pos.y + right.y * lateral,
            pos.z,
        );

        let clamped = seq.min(max_idx) as u8;
        output.push(make_object(config.mode, offset_pos, heading, floating, clamped));

        seq += 1;
        current_distance += config.row_spacing;
    }

    Ok(output)
}
