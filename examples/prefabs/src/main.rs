//! Prefab toolbox for LFS layout editing.
use std::{fmt, net::SocketAddr, path::PathBuf, time::Duration};

use clap::Parser;
use insim::{
    Packet, WithRequestId,
    core::heading::Heading,
    identifiers::{ConnectionId, RequestId},
    insim::{Axm, BfnType, ObjectInfo, PmoAction, PmoFlags, TinyType, TtcType},
};
use kitcar::ui::{Canvas, Component};
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

    /// Path to prefabs.yaml
    #[arg(long)]
    prefabs: PathBuf,
}

#[derive(Debug)]
struct State {
    prefabs: tools::prefabs::Prefabs,
    selection: Vec<ObjectInfo>,
    ui_visible: bool,
    nudge_distance_metres: f64,
    compass_visible: bool,
    compass_text: Option<String>,
}

#[derive(Debug)]
enum Command {
    ReloadPrefabs,
    SavePrefabs(String),
    SpawnObjects {
        objects: Vec<ObjectInfo>,
        action: PmoAction,
        origin: SpawnOrigin,
    },
}

#[derive(Debug, Clone, Copy)]
enum SpawnOrigin {
    Prefab,
    PaintedText,
    SplineDistrib {
        spacing_metres: f64,
    },
    Rotate {
        degrees: f64,
    },
    Nudge {
        heading: Heading,
        distance_metres: f64,
    },
    JiggleSelection,
}

impl fmt::Display for SpawnOrigin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SpawnOrigin::Prefab => write!(f, "prefab"),
            SpawnOrigin::PaintedText => write!(f, "painted text"),
            SpawnOrigin::SplineDistrib { spacing_metres } => {
                write!(f, "spline distribution ({spacing_metres}m spacing)")
            },
            SpawnOrigin::Rotate { degrees } => write!(f, "rotation ({degrees} degrees)"),
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
        }
    }
}

fn reduce_toolbox_message(state: &mut State, msg: ui::ToolboxMsg) -> Option<Command> {
    match msg {
        ui::ToolboxMsg::ExpandToolboxSection(_) => None,
        ui::ToolboxMsg::ReloadYaml => Some(Command::ReloadPrefabs),
        ui::ToolboxMsg::SavePrefab(name) => Some(Command::SavePrefabs(name.trim().to_string())),
        ui::ToolboxMsg::SpawnPrefab(idx) => {
            let Some(prefab) = state.prefabs.data.get(idx) else {
                return None;
            };

            let anchor = state
                .selection
                .first()
                .map(|obj| *obj.position())
                .unwrap_or_default();

            Some(Command::SpawnObjects {
                objects: prefab.place_at_anchor(anchor),
                action: PmoAction::Selection,
                origin: SpawnOrigin::Prefab,
            })
        },
        ui::ToolboxMsg::PaintedTextInput(text) => {
            let text = text.trim().to_string();
            if text.is_empty() {
                tracing::warn!("paint skipped: text input is empty");
                return None;
            }

            let anchor = state
                .selection
                .first()
                .map(|obj| *obj.position())
                .unwrap_or_default();
            let heading = state
                .selection
                .first()
                .and_then(ObjectInfo::heading)
                .unwrap_or_default();
            let objects = tools::painted_letters::build(&text, anchor, heading);

            if objects.is_empty() {
                tracing::warn!("paint skipped: text has no supported painted-letter characters");
                None
            } else {
                Some(Command::SpawnObjects {
                    objects,
                    action: PmoAction::Selection,
                    origin: SpawnOrigin::PaintedText,
                })
            }
        },
        ui::ToolboxMsg::SplineDistribInput(input) => {
            let trimmed = input.trim();

            if trimmed.is_empty() {
                tracing::warn!("spacing skipped: input is empty");
                return None;
            }

            match trimmed.parse::<f64>() {
                Ok(value) if value > 0.0 => {
                    match tools::spline_distrib::build(&state.selection, value, None) {
                        Ok(objects) => Some(Command::SpawnObjects {
                            objects,
                            action: PmoAction::AddObjects,
                            origin: SpawnOrigin::SplineDistrib {
                                spacing_metres: value,
                            },
                        }),
                        Err(err) => {
                            tracing::warn!("spacing skipped: {err}");
                            None
                        },
                    }
                },
                Ok(_) => {
                    tracing::warn!("spacing skipped: value must be greater than zero");
                    None
                },
                Err(_) => {
                    tracing::warn!("spacing skipped: input is not a number");
                    None
                },
            }
        },
        ui::ToolboxMsg::RotateInput(input) => {
            let trimmed = input.trim();

            if trimmed.is_empty() {
                tracing::warn!("rotation skipped: input is empty");
                return None;
            }

            match trimmed.parse::<f64>() {
                Ok(value) if value.is_finite() => {
                    match tools::rotate::build(&state.selection, value) {
                        Ok(objects) => Some(Command::SpawnObjects {
                            objects,
                            action: PmoAction::AddObjects,
                            origin: SpawnOrigin::Rotate { degrees: value },
                        }),
                        Err(err) => {
                            tracing::warn!("rotation skipped: {err}");
                            None
                        },
                    }
                },
                Ok(_) => {
                    tracing::warn!("rotation skipped: value must be finite");
                    None
                },
                Err(_) => {
                    tracing::warn!("rotation skipped: input is not a number");
                    None
                },
            }
        },
        ui::ToolboxMsg::NudgeDistanceInput(input) => {
            let trimmed = input.trim();

            if trimmed.is_empty() {
                tracing::warn!("nudge skipped: input is empty");
                return None;
            }

            match trimmed.parse::<f64>() {
                Ok(value) if value.is_finite() && value > 0.0 => {
                    state.nudge_distance_metres = value;
                    tracing::info!("Set nudge distance to {value} metres");
                },
                Ok(_) => {
                    tracing::warn!("nudge skipped: value must be finite and greater than zero");
                },
                Err(_) => {
                    tracing::warn!("nudge skipped: input is not a number");
                },
            }

            None
        },
        ui::ToolboxMsg::Nudge(heading) => Some(Command::SpawnObjects {
            objects: tools::nudge::nudge(
                &state.selection,
                heading.clone(),
                state.nudge_distance_metres,
            ),
            action: PmoAction::AddObjects,
            origin: SpawnOrigin::Nudge {
                heading,
                distance_metres: state.nudge_distance_metres,
            },
        }),
        ui::ToolboxMsg::JiggleSelection => {
            if state.selection.is_empty() {
                tracing::warn!("jiggle skipped: selection is empty");
                None
            } else {
                Some(Command::SpawnObjects {
                    objects: tools::jiggle::jiggle(&state.selection, 5.0, 3.5),
                    action: PmoAction::AddObjects,
                    origin: SpawnOrigin::JiggleSelection,
                })
            }
        },
    }
}

fn reduce_options_message(state: &mut State, msg: ui::OptionsMsg) -> Option<Command> {
    match msg {
        ui::OptionsMsg::ToggleSelectionInfo => None,
        ui::OptionsMsg::ToggleCompass => {
            state.compass_visible = !state.compass_visible;
            if !state.compass_visible {
                state.compass_text = None;
            }
            tracing::info!(
                "Compass {}",
                if state.compass_visible {
                    "enabled"
                } else {
                    "disabled"
                }
            );

            None
        },
    }
}

fn reduce_ui_message(state: &mut State, msg: ui::PrefabViewMessage) -> Option<Command> {
    match msg {
        ui::PrefabViewMessage::TopTab(_) => None,
        ui::PrefabViewMessage::Toolbox(toolbox_msg) => reduce_toolbox_message(state, toolbox_msg),
        ui::PrefabViewMessage::Options(options_msg) => reduce_options_message(state, options_msg),
    }
}

async fn run_command(
    connection: &mut FramedConnection,
    state: &mut State,
    command: Command,
) -> anyhow::Result<()> {
    match command {
        Command::ReloadPrefabs => {
            let path = state.prefabs.path.clone();
            state.prefabs = tools::prefabs::Prefabs::load(path)?;
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
    state: &State,
    objects: Vec<ObjectInfo>,
    pmoaction: PmoAction,
) -> insim::Result<usize> {
    if objects.is_empty() {
        return Ok(0);
    }

    if !state.selection.is_empty() {
        connection
            .write(Axm {
                ucid: ConnectionId::LOCAL,
                pmoaction: PmoAction::DelObjects,
                info: state.selection.clone(),
                ..Default::default()
            })
            .await?;
    }

    for chunk in objects.chunks(60) {
        connection
            .write(Axm {
                ucid: ConnectionId::LOCAL,
                pmoaction: pmoaction.clone(),
                info: chunk.to_vec(),
                ..Default::default()
            })
            .await?;
    }

    if matches!(pmoaction, PmoAction::AddObjects) {
        sleep(Duration::from_millis(50)).await;
        for chunk in objects.chunks(60) {
            connection
                .write(Axm {
                    ucid: ConnectionId::LOCAL,
                    pmoaction: PmoAction::Selection,
                    pmoflags: PmoFlags::SELECTION_REAL,
                    info: chunk.to_vec(),
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
        compass_visible: false,
        nudge_distance_metres: 1.0,
        compass_text: None,
    };

    let mut view = ui::PrefabView::default();
    let mut canvas = Canvas::<ui::PrefabViewMessage>::new(ConnectionId::LOCAL);
    let mut blocked = false;
    let mut dirty = true;
    let mut camera_tick = tokio::time::interval(Duration::from_millis(100));
    camera_tick.set_missed_tick_behavior(MissedTickBehavior::Skip);

    connection
        .write(TinyType::Sst.with_request_id(REQI_STATE))
        .await?;

    loop {
        if dirty && !blocked {
            if let Some(diff) = canvas.reconcile(
                view.render(ui::PrefabViewProps {
                    ui_visible: state.ui_visible,
                    selection_count: state.selection.len(),
                    prefabs: state
                        .prefabs
                        .data
                        .iter()
                        .map(|prefab| ui::PrefabListItem {
                            name: prefab.name.clone(),
                            count: prefab.objects.len(),
                        })
                        .collect(),
                    nudge_distance_metres: state.nudge_distance_metres,
                    compass_visible: state.compass_visible,
                    compass_text: state.compass_text.clone(),
                }),
            ) {
                for packet in diff.merge() {
                    connection.write(packet).await?;
                }
            }
            dirty = false;
        }

        tokio::select! {
            _ = camera_tick.tick(), if state.compass_visible => {
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
                        if matches!(axm.pmoaction, PmoAction::TtcSel) && axm.reqi == REQI_SELECTION {
                            state.selection = axm.info;
                            dirty = true;
                        }

                        None
                    },
                    Packet::Btc(btc) => {
                        if let Some(msg) = canvas.translate_clickid(&btc.clickid) {
                            view.update(msg.clone());
                            dirty = true;
                            Some(msg)
                        } else {
                            None
                        }

                    },
                    Packet::Btt(btt) => {
                        if let Some(msg) = canvas.translate_typein_clickid(&btt.clickid, btt.text) {
                            view.update(msg.clone());
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
                let Some(command) = reduce_ui_message(&mut state, msg) {
                    run_command(&mut connection, &mut state, command).await?;
                }
            }
        }
    }

    Ok(())
}
