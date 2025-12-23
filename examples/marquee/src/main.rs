use std::{
    net::SocketAddr,
    time::{Duration, Instant},
};

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Debug, Subcommand)]
enum Mode {
    WhereTheFuckIAm,
    Marquee {
        x: f64,
        y: f64,
        z: f64,
        heading: f64,
        text: String,
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

use glam::{DVec3, I16Vec3, IVec3, Vec3};
use insim::{
    core::{
        coordinate::Coordinate, direction::Heading, object::{
            /* painted::{Character, Letters, PaintColour},  */ letterboard_rb::{Character, LetterboardRB, LetterboardRBColour}, ObjectCoordinate, ObjectInfo, ObjectKind
        }
    }, identifiers::PlayerId, insim::{Axm, PmoAction, TinyType}, Packet, WithRequestId
};

fn position_text(
    text: &str,
    center: DVec3,
    heading: Heading,
    spacing_meters: f64,
    forward_meters: f64,
    elapsed: Duration,
) -> Vec<ObjectInfo> {
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

    text.chars()
        .enumerate()
        .flat_map(|(i, ch)| {
            let letter = Character::try_from(ch).ok()?;

            let base_off = (i as f64 * spacing) - scroll;
            let final_off =
                (base_off + total_width + total_width / 2.0) % total_width - total_width / 2.0;

            if final_off.abs() > view_limit {
                return None;
            }

            Some(ObjectInfo {
                xyz: ObjectCoordinate::from_dvec3_metres(anchor + (right * final_off)),
                kind: ObjectKind::LetterboardRB(LetterboardRB {
                    character: letter,
                    // XXX: for letterboards we need to rotate by 180deg, for painted characters
                    // they're fine as is.
                    heading: Heading::from_radians(heading.to_radians() + std::f64::consts::PI),
                    colour: LetterboardRBColour::Red,
                    floating: false,
                }),
            })
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

    match args.mode {
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

                            println!("{} {} {} {}", pos.x, pos.y, pos.z, comp_car.heading.to_radians());
                            break;
                        }
                    },

                    _ => {},
                }
            }
        },
        Mode::Marquee {
            x,
            y,
            z,
            heading,
            text,
        } => {
            let started = Instant::now();
            let mut interval = tokio::time::interval(Duration::from_millis(1000));
            let center = DVec3::new(x, y, z);
            let dir = Heading::from_radians(heading);
            let mut last_objects: Vec<ObjectInfo> = vec![];

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

                        last_objects = position_text(&text, center, dir, 1.0, 10.0, started.elapsed());

                        connection.write(Axm {
                            pmoaction: PmoAction::AddObjects,
                            info: last_objects.clone(),
                            ..Default::default()
                        }).await?;
                    }
                }
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
        },
    }

    Ok(())
}
