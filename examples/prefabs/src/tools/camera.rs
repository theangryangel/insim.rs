use std::time::Duration;

use insim::{
    core::heading::{HeadingU16, ObjectHeading},
    identifiers::{PlayerId, RequestId},
    insim::{Cpp, ObjectInfo, StaFlags},
};

pub fn get_top_down_view(selection: &[ObjectInfo], last_cpp: &Cpp) -> Option<Cpp> {
    let target = get_target(selection)?;
    let pos_m = target.position().xyz_metres();

    // Look down. Heading matches object heading (ObjectHeading -> HeadingU16 via degrees)
    let target_degrees = target.heading().map(|h| h.to_degrees()).unwrap_or(0.0);

    let mut cpp = Cpp {
        reqi: RequestId(0),
        h: HeadingU16::from_degrees(target_degrees),
        // Pitch 90 degrees down. 65536 = 360 deg. 90 deg = 16384.
        p: 16384,
        r: 0,
        viewplid: PlayerId(0),
        ingamecam: last_cpp.ingamecam.clone(),
        fov: last_cpp.fov,                // Use current FOV
        time: Duration::from_millis(500), // Smooth transition
        flags: StaFlags::SHIFTU,
        ..Default::default()
    };

    // Position camera at target X, Y, but fixed height 100m
    cpp.pos.x = (pos_m.0 * 65536.0) as i32;
    cpp.pos.y = (pos_m.1 * 65536.0) as i32;
    cpp.pos.z = (100.0 * 65536.0) as i32;

    Some(cpp)
}

pub fn get_side_view(selection: &[ObjectInfo], last_cpp: &Cpp) -> Option<Cpp> {
    let target = get_target(selection)?;
    let pos_m = target.position().xyz_metres();
    let heading = target.heading().unwrap_or(ObjectHeading::NORTH);

    // Place camera 90 degrees to the left of the object heading, 5m away
    // Heading + 90 degrees
    let side_angle = heading.to_radians() + std::f64::consts::FRAC_PI_2;

    let dist = 5.0;
    let offset_x = -side_angle.sin() * dist;
    let offset_y = side_angle.cos() * dist;

    let cam_x = pos_m.0 as f64 + offset_x;
    let cam_y = pos_m.1 as f64 + offset_y;
    let cam_z = pos_m.2 as f64; // Align to height of object

    // Camera look direction: Look back at object.
    let cam_h = HeadingU16::from_radians(heading.to_radians() - std::f64::consts::FRAC_PI_2);

    let mut cpp = Cpp {
        reqi: RequestId(0),
        h: cam_h,
        p: 0, // Level
        r: 0,
        viewplid: PlayerId(0),
        ingamecam: last_cpp.ingamecam.clone(),
        fov: last_cpp.fov, // Use current FOV
        time: Duration::from_millis(500),
        flags: StaFlags::SHIFTU,
        ..Default::default()
    };
    cpp.pos.x = (cam_x * 65536.0) as i32;
    cpp.pos.y = (cam_y * 65536.0) as i32;
    cpp.pos.z = (cam_z * 65536.0) as i32;

    Some(cpp)
}

fn get_target(selection: &[ObjectInfo]) -> Option<&ObjectInfo> {
    if selection.is_empty() {
        return None;
    }

    if selection.len() == 1 {
        return selection.first();
    }

    // Calculate centroid
    let mut sum_x = 0.0;
    let mut sum_y = 0.0;
    for obj in selection {
        let pos = obj.position().xyz_metres();
        sum_x += pos.0;
        sum_y += pos.1;
    }

    let avg_x = sum_x / selection.len() as f32;
    let avg_y = sum_y / selection.len() as f32;

    // Find object closest to centroid (ignoring Z for "central")
    selection.iter().min_by(|a, b| {
        let pos_a = a.position().xyz_metres();
        let pos_b = b.position().xyz_metres();

        let dist_a = (pos_a.0 - avg_x).powi(2) + (pos_a.1 - avg_y).powi(2);
        let dist_b = (pos_b.0 - avg_x).powi(2) + (pos_b.1 - avg_y).powi(2);

        dist_a
            .partial_cmp(&dist_b)
            .unwrap_or(std::cmp::Ordering::Equal)
    })
}
