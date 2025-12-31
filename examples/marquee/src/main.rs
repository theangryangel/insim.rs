use std::{
    net::SocketAddr,
    time::{Duration, Instant},
};

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Debug, Subcommand)]
enum Mode {
    WhereTheFuckIAm,
    Letterboard {
        x: f64,
        y: f64,
        z: f64,
        heading: f64,
        text: String,
    },
    Painted {
        x: f64,
        y: f64,
        z: f64,
        heading: f64,
        text: String,
    },
    Circle {
        x: f64,
        y: f64,
        z: f64,
        heading: f64,
        radius: f64,
        count: u8,
    },
    Signal {
        x: f64,
        y: f64,
        z: f64,
        heading: f64,
    },
}

/// Marquee
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[arg(long)]
    /// host:port of LFS to connect to
    addr: SocketAddr,

    #[command(subcommand)]
    mode: Mode,
}

use glam::DVec3;
use insim::{
    Packet, WithRequestId,
    core::{
        heading::Heading,
        object::{ObjectCoordinate, ObjectInfo, chalk, concrete, letterboard_rb, painted, tyres},
    },
    identifiers::PlayerId,
    insim::{Axm, PmoAction, TinyType},
};

fn position_text_common(
    text: &str,
    center: DVec3,
    heading: Heading,
    spacing_meters: f64,
    forward_meters: f64,
    elapsed: Duration,
) -> impl Iterator<Item = (usize, char, DVec3, Heading)> {
    const SPEED: f64 = 1.0;
    const MAX_VISIBLE: f64 = 10.0;
    const PADDING: usize = 2;

    let rad = heading.to_radians();
    let forward = DVec3::new(-rad.sin(), rad.cos(), 0.0);
    let right = DVec3::new(forward.y, -forward.x, 0.0);
    let anchor = center + (forward * forward_meters);
    let spacing = spacing_meters;
    let total_width = (text.len() + PADDING) as f64 * spacing;
    let scroll = (elapsed.as_secs_f64() * SPEED) % total_width;
    let view_limit = (MAX_VISIBLE * spacing) / 2.0;

    text.chars().enumerate().filter_map(move |(i, ch)| {
        let base_off = (i as f64 * spacing) - scroll;
        let final_off =
            (base_off + total_width + total_width / 2.0) % total_width - total_width / 2.0;
        if final_off.abs() > view_limit {
            return None;
        }
        let position = anchor + (right * final_off);
        Some((i, ch, position, heading))
    })
}

fn position_letterboard(
    text: &str,
    center: DVec3,
    heading: Heading,
    spacing_meters: f64,
    forward_meters: f64,
    elapsed: Duration,
) -> Vec<ObjectInfo> {
    if text.is_empty() {
        return Vec::new();
    }

    position_text_common(
        text,
        center,
        heading,
        spacing_meters,
        forward_meters,
        elapsed,
    )
    .flat_map(|(_, ch, position, heading)| {
        let letter = letterboard_rb::Character::try_from(ch).ok()?;
        Some(ObjectInfo::LetterboardRB(letterboard_rb::LetterboardRB {
            xyz: ObjectCoordinate::from_dvec3_metres(position),
            character: letter,
            heading: Heading::from_radians(heading.to_radians() + std::f64::consts::PI),
            colour: letterboard_rb::LetterboardRBColour::Red,
            floating: false,
        }))
    })
    .collect()
}

fn position_painted(
    text: &str,
    center: DVec3,
    heading: Heading,
    spacing_meters: f64,
    forward_meters: f64,
    elapsed: Duration,
) -> Vec<ObjectInfo> {
    if text.is_empty() {
        return Vec::new();
    }

    position_text_common(
        text,
        center,
        heading,
        spacing_meters,
        forward_meters,
        elapsed,
    )
    .flat_map(|(_, ch, position, heading)| {
        let letter = painted::Character::try_from(ch).ok()?;
        Some(ObjectInfo::PaintLetters(painted::Letters {
            xyz: ObjectCoordinate::from_dvec3_metres(position),
            character: letter,
            heading: Heading::from_radians(heading.to_radians()),
            colour: painted::PaintColour::Yellow,
            floating: false,
        }))
    })
    .collect()
}

fn position_tyrestack_circle(
    center: DVec3,
    heading: Heading,
    radius_metres: f64,
    elapsed: Duration,
    max: u8,
) -> Vec<ObjectInfo> {
    if max == 0 {
        return Vec::new();
    }

    let time_offset = elapsed.as_secs_f64();
    let angle_step = std::f64::consts::TAU / max as f64;

    (0..max)
        .map(|i| {
            let base_angle = i as f64 * angle_step;
            let x = center.x + (radius_metres as f64) * base_angle.cos();
            let y = center.y + (radius_metres as f64) * base_angle.sin();
            let position = DVec3::new(x, y, center.z);

            // Rotate color pattern based on time (completes full rotation every max seconds)
            let color_index = ((i as f64 + time_offset) % max as f64).floor() as u8;
            let is_dark = color_index % 2 == 0;

            ObjectInfo::TyreStack4(tyres::Tyres {
                xyz: ObjectCoordinate::from_dvec3_metres(position),
                colour: if is_dark {
                    tyres::TyreColour::Blue
                } else {
                    tyres::TyreColour::Red
                },
                floating: false,
                heading,
            })
        })
        .collect()
}

pub fn generate_checkpoint_signal(location: DVec3, heading: Heading) -> Vec<ObjectInfo> {
    use std::f64::consts::FRAC_PI_2;

    // Structure: (Local Position, The Object with Local Rotation)
    // Local Rotation: 0.0 for pillars, -90 deg (-PI/2) for arms (facing right)
    // FIXME: we can probably just move the DVec3 into the ObjectInfo now. But lazy.
    let parts = vec![
        // Pillars
        (
            DVec3::new(0.0, 0.0, -0.25),
            ObjectInfo::ConcretePillar(concrete::ConcretePillar {
                xyz: ObjectCoordinate::default(),
                x: concrete::Size::ThreeQuarter,
                y: concrete::Size::ThreeQuarter,
                height: concrete::ConcreteHeight::M4_00,
                heading: Heading::from_radians(0.0),
            }),
        ),
        (
            DVec3::new(0.0, 0.0, 3.75),
            ObjectInfo::ConcretePillar(concrete::ConcretePillar {
                xyz: ObjectCoordinate::default(),
                x: concrete::Size::ThreeQuarter,
                y: concrete::Size::ThreeQuarter,
                height: concrete::ConcreteHeight::M2_25,
                heading: Heading::from_radians(0.0),
            }),
        ),
        // Arms (Offset 1.70m Right, Rotated -90 deg) --
        (
            DVec3::new(1.70, 0.0, 3.70),
            ObjectInfo::ConcreteSlabWall(concrete::ConcreteSlabWall {
                xyz: ObjectCoordinate::default(),
                colour: concrete::ConcreteColour::Yellow,
                length: concrete::ConcreteWidthLength::Four,
                pitch: concrete::ConcretePitch::Deg42,
                heading: Heading::from_radians(-FRAC_PI_2), // Facing Right
            }),
        ),
        (
            DVec3::new(1.70, 0.0, 4.70),
            ObjectInfo::ConcreteSlabWall(concrete::ConcreteSlabWall {
                xyz: ObjectCoordinate::default(),
                colour: concrete::ConcreteColour::Red,
                length: concrete::ConcreteWidthLength::Four,
                pitch: concrete::ConcretePitch::Deg42,
                heading: Heading::from_radians(-FRAC_PI_2),
            }),
        ),
        (
            DVec3::new(1.70, 0.0, 5.70),
            ObjectInfo::ConcreteSlabWall(concrete::ConcreteSlabWall {
                xyz: ObjectCoordinate::default(),
                colour: concrete::ConcreteColour::Blue,
                length: concrete::ConcreteWidthLength::Four,
                pitch: concrete::ConcretePitch::Deg42,
                heading: Heading::from_radians(-FRAC_PI_2),
            }),
        ),
        // Chalk line on floor
        (
            DVec3::new(4.5, 0.0, 0.00),
            ObjectInfo::ChalkLine(chalk::Chalk {
                xyz: ObjectCoordinate::default(),
                heading: heading,
                colour: chalk::ChalkColour::Yellow,
                floating: false,
            }),
        ),
        // Tyres at end of chalk
        (
            DVec3::new(9.0, 0.0, 0.00),
            ObjectInfo::TyreStack4Big(tyres::Tyres {
                xyz: ObjectCoordinate::default(),
                colour: tyres::TyreColour::Yellow,
                heading: heading,
                floating: false,
            }),
        ),
        (
            DVec3::new(9.0, 0.0, 0.75),
            ObjectInfo::TyreStack4Big(tyres::Tyres {
                xyz: ObjectCoordinate::default(),
                colour: tyres::TyreColour::Yellow,
                heading: heading,
                floating: true,
            }),
        ),
    ];

    // --- Calculate World Matrix (ACW) ---
    let global_rad = heading.to_radians();
    let (sin, cos) = global_rad.sin_cos();

    // Basis Vectors (ACW System: 0=N, 90=W)
    let fwd_x = -sin;
    let fwd_y = cos;
    let right_x = cos; // 90 deg "Starboard"
    let right_y = sin;

    // --- Transform & Build ---
    parts
        .into_iter()
        .map(|(local_pos, mut kind)| {
            // A. Transform Position
            // WorldPos = Origin + (Right * x) + (Fwd * y) + (Up * z)
            let world_pos = DVec3::new(
                location.x + (right_x * local_pos.x) + (fwd_x * local_pos.y),
                location.y + (right_y * local_pos.x) + (fwd_y * local_pos.y),
                location.z + local_pos.z,
            );

            // B. Transform Rotation
            // We update the 'kind' in place by adding the global heading to its local heading
            match &mut kind {
                ObjectInfo::ConcretePillar(p) => {
                    p.heading = Heading::from_radians(global_rad + p.heading.to_radians());
                },
                ObjectInfo::ConcreteSlabWall(w) => {
                    w.heading = Heading::from_radians(global_rad + w.heading.to_radians());
                },
                _ => {}, // Handle other types if necessary
            }
            *kind.position_mut() = ObjectCoordinate::from_dvec3_metres(world_pos);
            kind
        })
        .collect()
}

fn setup_tracing_subscriber() {
    // Setup with a default log level of INFO RUST_LOG is unset
    tracing_subscriber::fmt::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive(tracing_subscriber::filter::LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .init();
}

#[tokio::main]
pub async fn main() -> Result<()> {
    setup_tracing_subscriber();

    let args = Cli::parse();

    let mut connection = insim::tcp(args.addr)
        .isi_flag_local(true)
        .isi_iname(Some("paint".to_string()))
        .isi_flag_axm_edit(true)
        .connect_async()
        .await?;

    let mut last_objects: Vec<ObjectInfo> = vec![];

    match &args.mode {
        Mode::WhereTheFuckIAm => {
            connection.write(TinyType::Sst.with_request_id(1)).await?;
            connection.write(TinyType::Mci.with_request_id(2)).await?;
            let mut viewed = PlayerId(0);
            loop {
                match connection.read().await? {
                    Packet::Sta(sta) => {
                        viewed = sta.viewplid;
                    },

                    Packet::Mci(mci) => {
                        // Find the viewed player's position in MCI
                        if let Some(comp_car) = mci.info.iter().next() {
                            if comp_car.plid != viewed {
                                continue;
                            }
                            let pos = comp_car.xyz.to_dvec3_metres();

                            println!(
                                "{} {} {} {}",
                                pos.x,
                                pos.y,
                                pos.z,
                                comp_car.heading.to_radians()
                            );
                            break;
                        }
                    },

                    _ => {},
                }
            }
        },
        Mode::Signal { x, y, z, heading } => {
            let center = DVec3::new(*x, *y, *z);
            last_objects = generate_checkpoint_signal(center, Heading::from_radians(*heading));

            connection
                .write(Axm {
                    pmoaction: PmoAction::AddObjects,
                    info: last_objects.clone(),
                    ..Default::default()
                })
                .await?;

            loop {
                tokio::select! {
                    _ = tokio::signal::ctrl_c() => {
                        break;
                    },
                    res = connection.read() => {
                        res?;
                    },
                }
            }
        },
        _ => {
            let started = Instant::now();
            let mut interval = tokio::time::interval(Duration::from_millis(1000));

            loop {
                tokio::select! {
                    _ = tokio::signal::ctrl_c() => {
                        break;
                    },
                    res = connection.read() => {
                        res?;
                    },
                    _ = interval.tick() => {
                        if !last_objects.is_empty() {
                            connection.write(Axm {
                                pmoaction: PmoAction::DelObjects,
                                info: last_objects,
                                ..Default::default()
                            }).await?;
                        }

                        match &args.mode {
                            Mode::Letterboard {
                                x,
                                y,
                                z,
                                heading,
                                text,
                            } => {
                                let center = DVec3::new(*x, *y, *z);
                                let dir = Heading::from_radians(*heading);
                                last_objects = position_letterboard(&text, center, dir, 1.0, 10.0, started.elapsed());
                            },
                            Mode::Painted {
                                x, y, z, heading, text
                            } => {
                                let center = DVec3::new(*x, *y, *z);
                                let dir = Heading::from_radians(*heading);
                                last_objects = position_painted(&text, center, dir, 1.0, 10.0, started.elapsed());
                            },
                            Mode::Circle { x, y, z, heading, radius, count } => {
                                let center = DVec3::new(*x, *y, *z);
                                let dir = Heading::from_radians(*heading);
                                last_objects = position_tyrestack_circle(center, dir, *radius, started.elapsed(), *count);
                            },
                            _ => {
                                unreachable!()
                            }
                        }

                        connection.write(Axm {
                            pmoaction: PmoAction::AddObjects,
                            info: last_objects.clone(),
                            ..Default::default()
                        }).await?;
                    }
                }
            }
        },
    }

    if !last_objects.is_empty() {
        let _ = connection
            .write(Axm {
                pmoaction: PmoAction::DelObjects,
                info: last_objects,
                ..Default::default()
            })
            .await?;
        // wait for lfs to respond to the clean up... eeh. not a fan of this hack.
        // but it seems to be how LFS processes things
        let _ = tokio::time::sleep(Duration::from_millis(200)).await;
        connection.shutdown().await?;
    }

    Ok(())
}
