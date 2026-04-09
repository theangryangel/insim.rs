//! Prefab toolbox for LFS layout editing.
use std::{fmt, net::SocketAddr, path::PathBuf, time::Duration};

use clap::Parser;
use insim::{
    Packet, WithRequestId,
    core::heading::Heading,
    identifiers::{ConnectionId, RequestId},
    insim::{Axm, BfnType, Cpp, ObjectInfo, PmoAction, PmoFlags, TinyType, TtcType},
};
use insim_extras::ui::{Canvas, Component};
use tokio::time::{MissedTickBehavior, sleep};

mod tools;
mod ui;

const REQI_SELECTION: RequestId = RequestId(200);
const REQI_CAMERA: RequestId = RequestId(201);
const REQI_STATE: RequestId = RequestId(202);
const COMPASS_WIDTH: usize = 31;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[arg(long)]
    /// host:port of LFS to connect to
    addr: SocketAddr,

    /// Path to directory containing prefab YAML files (one file per prefab)
    #[arg(long)]
    prefabs: PathBuf,
}

#[derive(Debug)]
struct UndoEntry {
    deleted: Vec<ObjectInfo>,
    added: Vec<ObjectInfo>,
    selection_before: Vec<ObjectInfo>,
}

#[derive(Debug)]
struct State {
    prefabs: tools::prefabs::Prefabs,
    selection: Vec<ObjectInfo>,
    ui_visible: bool,
    display_selection_info: bool,
    nudge_distance_metres: f64,
    radial_count: usize,
    radial_radius_metres: f64,
    radial_arc_degrees: f64,
    ramp_mode: tools::ramp::RampMode,
    ramp_roll_degrees: f64,
    grid_mode: tools::grid::GridMode,
    grid_width: usize,
    grid_rows: usize,
    grid_col_spacing: f64,
    grid_row_spacing: f64,
    grid_lateral_offset: f64,
    compass_visible: bool,
    compass_text: Option<String>,
    last_cpp: Cpp,
    undo_stack: Vec<UndoEntry>,
}

#[derive(Debug, Clone, Copy)]
enum SpawnAction {
    AddObjects,
    Selection,
}

#[derive(Debug)]
enum Command {
    ReloadPrefabs,
    SavePrefabs(String),
    SpawnPrefab(usize),
    SpawnObjects {
        objects: Vec<ObjectInfo>,
        action: SpawnAction,
        origin: SpawnOrigin,
    },
    CameraMove(Cpp),
    Undo,
}

#[derive(Debug, Clone, Copy)]
enum SpawnOrigin {
    PaintedText,
    SplineDistrib {
        spacing_metres: f64,
    },
    Rotate {
        degrees: f64,
    },
    RotateEach {
        degrees: f64,
    },
    Ramp {
        mode: tools::ramp::RampMode,
        roll_degrees: f64,
    },
    Nudge {
        heading: Heading,
        distance_metres: f64,
    },
    JiggleSelection,
    Grid {
        mode: tools::grid::GridMode,
        width: usize,
        rows: usize,
    },
    Mirror {
        axis: tools::mirror::MirrorAxis,
    },
    RadialArray {
        count: usize,
        radius_metres: f64,
        arc_degrees: f64,
    },
}

impl fmt::Display for SpawnOrigin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SpawnOrigin::PaintedText => write!(f, "painted text"),
            SpawnOrigin::SplineDistrib { spacing_metres } => {
                write!(f, "spline distribution ({spacing_metres}m spacing)")
            },
            SpawnOrigin::Rotate { degrees } => write!(f, "rotation ({degrees} degrees)"),
            SpawnOrigin::RotateEach { degrees } => write!(f, "rotate each ({degrees} degrees)"),
            SpawnOrigin::Ramp { mode, roll_degrees } => match mode {
                tools::ramp::RampMode::AlongPath => {
                    write!(f, "concrete ramp/slab blend (grade along path)")
                },
                tools::ramp::RampMode::AcrossPath => {
                    write!(f, "concrete slab banking (roll {} degrees)", roll_degrees)
                },
            },
            SpawnOrigin::Nudge {
                heading,
                distance_metres,
            } => {
                let heading = if *heading == Heading::NORTH {
                    "north"
                } else if *heading == Heading::SOUTH {
                    "south"
                } else if *heading == Heading::EAST {
                    "east"
                } else if *heading == Heading::WEST {
                    "west"
                } else {
                    "unknown"
                };
                write!(f, "nudge {heading} by {distance_metres} metres")
            },
            SpawnOrigin::JiggleSelection => write!(f, "jiggle selection"),
            SpawnOrigin::Mirror { axis } => match axis {
                tools::mirror::MirrorAxis::X => write!(f, "mirror across X axis (left/right)"),
                tools::mirror::MirrorAxis::Y => write!(f, "mirror across Y axis (front/back)"),
            },
            SpawnOrigin::RadialArray {
                count,
                radius_metres,
                arc_degrees,
            } => write!(
                f,
                "radial array ({count} copies, {radius_metres}m radius, {arc_degrees}°)"
            ),
            SpawnOrigin::Grid { mode, width, rows } => {
                let mode_str = match mode {
                    tools::grid::GridMode::StartGrid => "start grid",
                    tools::grid::GridMode::Pit => "pit",
                    tools::grid::GridMode::PitBox => "pit box",
                };
                write!(f, "{mode_str} grid ({width}x{rows})")
            },
        }
    }
}

async fn run_command(
    connection: &mut FramedConnection,
    state: &mut State,
    command: Command,
) -> anyhow::Result<()> {
    match command {
        Command::ReloadPrefabs => {
            let dir = state.prefabs.dir.clone();
            state.prefabs = tools::prefabs::Prefabs::load(dir)?;
        },
        Command::SpawnPrefab(idx) => {
            let anchor = state
                .selection
                .first()
                .map(|obj| *obj.position())
                .unwrap_or_default();
            match state.prefabs.load_prefab(idx) {
                Ok(prefab) => {
                    let spawned = spawn_at_selection(
                        connection,
                        state,
                        prefab.place_at_anchor(anchor),
                        SpawnAction::AddObjects,
                    )
                    .await?;
                    if spawned > 0 {
                        tracing::info!("Spawned {spawned} objects (prefab)");
                    }
                },
                Err(err) => tracing::warn!("spawn skipped: {err}"),
            }
        },
        Command::SavePrefabs(name) => {
            match state
                .prefabs
                .add_and_save_selection(&name, &state.selection)
            {
                Ok(saved) => tracing::info!("Saved prefab '{saved}'"),
                Err(err) => tracing::warn!("save skipped: {err}"),
            }
        },
        Command::SpawnObjects {
            objects,
            action,
            origin,
        } => {
            let spawned = spawn_at_selection(connection, state, objects, action).await?;
            if spawned > 0 {
                tracing::info!("Spawned {spawned} objects ({origin})");
            }
        },
        Command::CameraMove(cpp) => {
            connection.write(cpp).await?;
        },
        Command::Undo => {
            if let Some(entry) = state.undo_stack.pop() {
                if !entry.added.is_empty() {
                    connection
                        .write(Axm {
                            ucid: ConnectionId::LOCAL,
                            action: PmoAction::DelObjects(entry.added),
                            ..Default::default()
                        })
                        .await?;
                }
                for chunk in entry.deleted.chunks(60) {
                    connection
                        .write(Axm {
                            ucid: ConnectionId::LOCAL,
                            action: PmoAction::AddObjects(chunk.to_vec()),
                            ..Default::default()
                        })
                        .await?;
                }
                if !entry.selection_before.is_empty() {
                    sleep(Duration::from_millis(50)).await;
                    for chunk in entry.selection_before.chunks(60) {
                        connection
                            .write(Axm {
                                ucid: ConnectionId::LOCAL,
                                flags: PmoFlags::SELECTION_REAL,
                                action: PmoAction::Selection(chunk.to_vec()),
                                ..Default::default()
                            })
                            .await?;
                    }
                }
            }
        },
    }

    Ok(())
}

type FramedConnection = insim::net::tokio_impl::Framed;

fn setup_tracing_subscriber() {
    tracing_subscriber::fmt::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive(tracing_subscriber::filter::LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .init();
}

fn clamp_i16(value: i32) -> i16 {
    value.clamp(i32::from(i16::MIN), i32::from(i16::MAX)) as i16
}

fn clamp_u8(value: i32) -> u8 {
    value.clamp(0, i32::from(u8::MAX)) as u8
}

async fn spawn_at_selection(
    connection: &mut FramedConnection,
    state: &mut State,
    objects: Vec<ObjectInfo>,
    action: SpawnAction,
) -> insim::Result<usize> {
    if objects.is_empty() {
        return Ok(0);
    }

    state.undo_stack.push(UndoEntry {
        deleted: state.selection.clone(),
        added: objects.clone(),
        selection_before: state.selection.clone(),
    });
    if state.undo_stack.len() > 20 {
        state.undo_stack.remove(0);
    }

    if !state.selection.is_empty() {
        connection
            .write(Axm {
                ucid: ConnectionId::LOCAL,
                action: PmoAction::DelObjects(state.selection.clone()),
                ..Default::default()
            })
            .await?;
    }

    for chunk in objects.chunks(60) {
        let axm_action = match action {
            SpawnAction::AddObjects => PmoAction::AddObjects(chunk.to_vec()),
            SpawnAction::Selection => PmoAction::Selection(chunk.to_vec()),
        };
        connection
            .write(Axm {
                ucid: ConnectionId::LOCAL,
                action: axm_action,
                ..Default::default()
            })
            .await?;
    }

    if matches!(action, SpawnAction::AddObjects) {
        sleep(Duration::from_millis(50)).await;
        for chunk in objects.chunks(60) {
            connection
                .write(Axm {
                    ucid: ConnectionId::LOCAL,
                    flags: PmoFlags::SELECTION_REAL,
                    action: PmoAction::Selection(chunk.to_vec()),
                    ..Default::default()
                })
                .await?;
        }
    }

    Ok(objects.len())
}

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    setup_tracing_subscriber();

    let cli = Cli::parse();
    let prefabs = tools::prefabs::Prefabs::load(cli.prefabs.clone())?;

    tracing::info!("Connecting via TCP to {}", &cli.addr);
    let mut connection = insim::tcp(cli.addr)
        .isi_iname(Some("prefab-toolbox".to_string()))
        .isi_flag_local(true)
        .isi_flag_axm_edit(true)
        .connect_async()
        .await?;

    tracing::info!("Connected");

    let mut state = State {
        prefabs,
        selection: Vec::new(),
        ui_visible: false,
        display_selection_info: true,
        compass_visible: false,
        nudge_distance_metres: 1.0,
        radial_count: 6,
        radial_radius_metres: 10.0,
        radial_arc_degrees: 360.0,
        ramp_mode: tools::ramp::RampMode::AlongPath,
        ramp_roll_degrees: 18.0,
        grid_mode: tools::grid::GridMode::StartGrid,
        grid_width: 2,
        grid_rows: 24,
        grid_col_spacing: 4.0,
        grid_row_spacing: 8.0,
        grid_lateral_offset: 3.0,
        compass_text: None,
        last_cpp: Cpp::default(),
        undo_stack: Vec::new(),
    };

    let mut ui_root = ui::Toolbox::default();

    let mut canvas = Canvas::<ui::ToolboxMsg>::new(ConnectionId::LOCAL);
    let mut blocked = false;
    let mut dirty = true;
    let mut camera_tick = tokio::time::interval(Duration::from_millis(100));
    camera_tick.set_missed_tick_behavior(MissedTickBehavior::Skip);

    connection
        .write(TinyType::Scp.with_request_id(REQI_CAMERA))
        .await?;

    connection
        .write(TinyType::Sst.with_request_id(REQI_STATE))
        .await?;

    loop {
        if dirty && !blocked {
            if let Some(diff) = canvas.reconcile(
                ui_root.render(ui::ToolboxProps {
                    ui_visible: state.ui_visible,
                    display_selection_info: state.display_selection_info,
                    selection_count: state.selection.len(),
                    prefabs: state
                        .prefabs
                        .entries
                        .iter()
                        .map(|entry| ui::PrefabSummary {
                            name: entry.name.clone(),
                        })
                        .collect(),
                    nudge_distance_metres: state.nudge_distance_metres,
                    radial_count: state.radial_count,
                    radial_radius_metres: state.radial_radius_metres,
                    radial_arc_degrees: state.radial_arc_degrees,
                    ramp_mode: state.ramp_mode,
                    ramp_roll_degrees: state.ramp_roll_degrees,
                    grid_mode: state.grid_mode,
                    grid_width: state.grid_width,
                    grid_rows: state.grid_rows,
                    grid_col_spacing: state.grid_col_spacing,
                    grid_row_spacing: state.grid_row_spacing,
                    grid_lateral_offset: state.grid_lateral_offset,
                    compass_visible: state.compass_visible,
                    compass_text: state.compass_text.clone(),
                    can_undo: !state.undo_stack.is_empty(),
                }),
            ) {
                for packet in diff.merge() {
                    connection.write(packet).await?;
                }
            }
            dirty = false;
        }

        tokio::select! {
            _ = camera_tick.tick() => {
                connection
                    .write(TinyType::Scp.with_request_id(REQI_CAMERA))
                    .await?;
            }
            _ = tokio::signal::ctrl_c() => {
                tracing::info!("Shutting down");
                break;
            }
            packet = connection.read() => {
                let packet = match packet {
                    Ok(packet) => packet,
                    Err(err) => {
                        tracing::error!("connection dropped: {err}");
                        break;
                    }
                };

                let msg = match packet {
                    Packet::Sta(sta) => {
                        state.last_cpp.ingamecam = sta.ingamecam;
                        let now_visible = sta.flags.is_shiftu() || sta.flags.is_shiftu_following();

                        if now_visible != state.ui_visible {
                            state.ui_visible = now_visible;
                            state.selection.clear();
                            dirty = true;

                            if now_visible {
                                connection
                                    .write(TtcType::SelStart.with_request_id(REQI_SELECTION))
                                    .await?;
                                connection
                                    .write(TtcType::Sel.with_request_id(REQI_SELECTION))
                                    .await?;
                            } else {
                                connection
                                    .write(TtcType::SelStop.with_request_id(REQI_SELECTION))
                                    .await?;
                            }
                        }

                        None
                    },
                    Packet::Cpp(cpp) => {
                        state.last_cpp = cpp.clone();
                        if state.compass_visible {
                            let next = Some(tools::compass::generate(cpp.h, COMPASS_WIDTH));

                            if state.compass_text != next {
                                state.compass_text = next;
                                dirty = true;
                            }
                        }
                        None
                    },
                    Packet::Axm(axm) => {
                        if axm.reqi == REQI_SELECTION {
                            if let PmoAction::TtcSel(info) = axm.action {
                                state.selection = info;
                                dirty = true;
                            }
                        }

                        None
                    },
                    Packet::Btc(btc) => {
                        if let Some(msg) = canvas.translate_clickid(&btc.clickid) {
                            ui_root.update(msg.clone());
                            dirty = true;
                            Some(msg)
                        } else {
                            None
                        }

                    },
                    Packet::Btt(btt) => {
                        if let Some(msg) = canvas.translate_typein_clickid(&btt.clickid, btt.text) {
                            ui_root.update(msg.clone());
                            dirty = true;
                            Some(msg)
                        } else {
                            None
                        }
                    },
                    Packet::Bfn(bfn) => {
                        match bfn.subt {
                            BfnType::Clear | BfnType::UserClear => {
                                blocked = true;
                                canvas.clear();
                            }
                            BfnType::BtnRequest => {
                                blocked = false;
                                dirty = true;
                            }
                            _ => {}
                        }

                        None
                    },
                    _ => {
                        None
                    }
                };

                if let Some(msg) = msg &&
                let Some(command) = ui::reduce_message(&mut state, msg) {
                    run_command(&mut connection, &mut state, command).await?;
                }
            }
        }
    }

    Ok(())
}
