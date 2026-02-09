//! Prefab toolbox for LFS layout editing.
use std::{net::SocketAddr, path::PathBuf, time::Duration};

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
    top_tab: ui::TopTab,
    expanded_section: ui::ExpandedSection,
    ui_visible: bool,
    display_selection_info: bool,
    nudge_distance_metres: f64,
    compass_visible: bool,
    compass_text: Option<String>,
}

async fn handle_ui_message(
    connection: &mut FramedConnection,
    state: &mut State,
    msg: ui::PrefabViewMessage,
) -> anyhow::Result<()> {
    match msg {
        ui::PrefabViewMessage::ShowToolboxTab => {
            state.top_tab = ui::TopTab::Toolbox;
        },
        ui::PrefabViewMessage::ShowOptionsTab => {
            state.top_tab = ui::TopTab::Options;
        },
        ui::PrefabViewMessage::TogglePrefabsSection => {
            state.expanded_section =
                if matches!(state.expanded_section, ui::ExpandedSection::Prefabs) {
                    ui::ExpandedSection::None
                } else {
                    ui::ExpandedSection::Prefabs
                };
        },
        ui::PrefabViewMessage::ToggleNudgeSection => {
            state.expanded_section = if matches!(state.expanded_section, ui::ExpandedSection::Nudge)
            {
                ui::ExpandedSection::None
            } else {
                ui::ExpandedSection::Nudge
            };
        },
        ui::PrefabViewMessage::ReloadYaml => {
            let path = state.prefabs.path.clone();
            state.prefabs = tools::prefabs::Prefabs::load(path)?;
        },
        ui::PrefabViewMessage::SavePrefab(name) => {
            let pending_name = name.trim().to_string();
            match state
                .prefabs
                .add_and_save_selection(&pending_name, &state.selection)
            {
                Ok(saved) => tracing::info!("Saved prefab '{saved}'"),
                Err(err) => tracing::warn!("save skipped: {err}"),
            }
        },
        ui::PrefabViewMessage::SpawnPrefab(idx) => {
            if let Some(prefab) = state.prefabs.data.get(idx) {
                let anchor = state
                    .selection
                    .first()
                    .map(|obj| *obj.position())
                    .unwrap_or_default();
                let _ = spawn_at_selection(
                    connection,
                    state,
                    prefab.place_at_anchor(anchor),
                    PmoAction::Selection,
                )
                .await?;
            }
        },
        ui::PrefabViewMessage::PaintedTextInput(text) => {
            let text = text.trim().to_string();

            if text.is_empty() {
                tracing::warn!("paint skipped: text input is empty");
            } else {
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
                let painted_text = tools::painted_letters::build(&text, anchor, heading);
                let painted_count =
                    spawn_at_selection(connection, state, painted_text, PmoAction::Selection)
                        .await?;

                if painted_count == 0 {
                    tracing::warn!(
                        "paint skipped: text has no supported painted-letter characters"
                    );
                } else {
                    tracing::info!("Painted {painted_count} letter objects into selection");
                }
            }
        },
        ui::PrefabViewMessage::SplineDistribInput(input) => {
            let trimmed = input.trim();

            if trimmed.is_empty() {
                tracing::warn!("spacing skipped: input is empty");
            } else {
                match trimmed.parse::<f64>() {
                    Ok(value) if value > 0.0 => {
                        match tools::spline_distrib::build(&state.selection, value, None) {
                            Ok(objects) => {
                                let placed = spawn_at_selection(
                                    connection,
                                    state,
                                    objects,
                                    PmoAction::AddObjects,
                                )
                                .await?;
                                tracing::info!("Placed {placed} spaced objects");
                            },
                            Err(err) => tracing::warn!("spacing skipped: {err}"),
                        }
                    },
                    Ok(_) => tracing::warn!("spacing skipped: value must be greater than zero"),
                    Err(_) => tracing::warn!("spacing skipped: input is not a number"),
                }
            }
        },
        ui::PrefabViewMessage::RotateInput(input) => {
            let trimmed = input.trim();

            if trimmed.is_empty() {
                tracing::warn!("rotation skipped: input is empty");
            } else {
                match trimmed.parse::<f64>() {
                    Ok(value) if value.is_finite() => {
                        match tools::rotate::build(&state.selection, value) {
                            Ok(objects) => {
                                let rotated = spawn_at_selection(
                                    connection,
                                    state,
                                    objects,
                                    PmoAction::AddObjects,
                                )
                                .await?;
                                tracing::info!("Rotated {rotated} objects by {value} degrees");
                            },
                            Err(err) => tracing::warn!("rotation skipped: {err}"),
                        }
                    },
                    Ok(_) => tracing::warn!("rotation skipped: value must be finite"),
                    Err(_) => tracing::warn!("rotation skipped: input is not a number"),
                }
            }
        },
        ui::PrefabViewMessage::NudgeDistanceInput(input) => {
            let trimmed = input.trim();

            if trimmed.is_empty() {
                tracing::warn!("nudge skipped: input is empty");
            } else {
                match trimmed.parse::<f64>() {
                    Ok(value) if value.is_finite() && value > 0.0 => {
                        state.nudge_distance_metres = value;
                        tracing::info!("Set nudge distance to {value} metres");
                    },
                    Ok(_) => {
                        tracing::warn!("nudge skipped: value must be finite and greater than zero")
                    },
                    Err(_) => tracing::warn!("nudge skipped: input is not a number"),
                }
            }
        },
        ui::PrefabViewMessage::NudgeNorth => {
            let nudged = tools::nudge::nudge(
                &state.selection,
                Heading::NORTH,
                state.nudge_distance_metres,
            );
            let moved =
                spawn_at_selection(connection, state, nudged.clone(), PmoAction::AddObjects)
                    .await?;
            tracing::info!(
                "Nudged {moved} objects north by {} metres",
                state.nudge_distance_metres
            );
        },
        ui::PrefabViewMessage::NudgeSouth => {
            let nudged = tools::nudge::nudge(
                &state.selection,
                Heading::SOUTH,
                state.nudge_distance_metres,
            );
            let moved =
                spawn_at_selection(connection, state, nudged.clone(), PmoAction::AddObjects)
                    .await?;
            tracing::info!(
                "Nudged {moved} objects south by {} metres",
                state.nudge_distance_metres
            );
        },
        ui::PrefabViewMessage::NudgeEast => {
            let nudged =
                tools::nudge::nudge(&state.selection, Heading::EAST, state.nudge_distance_metres);
            let moved =
                spawn_at_selection(connection, state, nudged.clone(), PmoAction::AddObjects)
                    .await?;
            tracing::info!(
                "Nudged {moved} objects east by {} metres",
                state.nudge_distance_metres
            );
        },
        ui::PrefabViewMessage::NudgeWest => {
            let nudged =
                tools::nudge::nudge(&state.selection, Heading::WEST, state.nudge_distance_metres);
            let moved =
                spawn_at_selection(connection, state, nudged.clone(), PmoAction::AddObjects)
                    .await?;
            tracing::info!(
                "Nudged {moved} objects west by {} metres",
                state.nudge_distance_metres
            );
        },
        ui::PrefabViewMessage::JiggleSelection => {
            if state.selection.is_empty() {
                tracing::warn!("jiggle skipped: selection is empty");
            } else {
                let jiggled = tools::jiggle::jiggle(&state.selection, 5.0, 3.5);
                let moved =
                    spawn_at_selection(connection, state, jiggled, PmoAction::AddObjects).await?;
                tracing::info!("Jiggled {moved} objects");
            }
        },
        ui::PrefabViewMessage::ToggleCompass => {
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
        },
        ui::PrefabViewMessage::ToggleSelectionInfo => {
            state.display_selection_info = !state.display_selection_info;
            tracing::info!(
                "Selection info display {}",
                if state.display_selection_info {
                    "enabled"
                } else {
                    "disabled"
                }
            );
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
        top_tab: ui::TopTab::Toolbox,
        expanded_section: ui::ExpandedSection::None,
        ui_visible: false,
        display_selection_info: true,
        compass_visible: false,
        nudge_distance_metres: 1.0,
        compass_text: None,
    };

    let view = ui::PrefabView;
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
                    top_tab: state.top_tab,
                    expanded_section: state.expanded_section,
                    ui_visible: state.ui_visible,
                    display_selection_info: state.display_selection_info,
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

                match packet {
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
                    },
                    Packet::Cpp(cpp) => if state.compass_visible {
                        let next = Some(tools::compass::generate(cpp.h, COMPASS_WIDTH));

                        if state.compass_text != next {
                            state.compass_text = next;
                            dirty = true;
                        }
                    },
                    Packet::Axm(axm) => {
                        if matches!(axm.pmoaction, PmoAction::TtcSel) && axm.reqi == REQI_SELECTION {
                            state.selection = axm.info;
                            dirty = true;
                        }
                    },
                    Packet::Btc(btc) => {
                        if let Some(msg) = canvas.translate_clickid(&btc.clickid) {
                            handle_ui_message(&mut connection, &mut state, msg).await?;
                            dirty = true;
                        }
                    },
                    Packet::Btt(btt) => {
                        if let Some(msg) = canvas.translate_typein_clickid(&btt.clickid, btt.text) {
                            handle_ui_message(&mut connection, &mut state, msg).await?;
                            dirty = true;
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
                    },
                    _ => {}
                }
            }
        }
    }

    Ok(())
}
