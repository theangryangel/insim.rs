//! Prefab toolbox for LFS layout editing.
use std::{net::SocketAddr, path::PathBuf};

use clap::Parser;
use insim::{
    Packet, WithRequestId,
    identifiers::{ConnectionId, RequestId},
    insim::{Axm, BfnType, BtnStyle, ObjectInfo, PmoAction, TtcType},
};
use kitcar::ui::{self, Canvas, Component};

mod tools;

const REQI_SELECTION: RequestId = RequestId(200);

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

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
enum ActiveTab {
    #[default]
    Prefabs,
    Tools,
}

#[derive(Debug)]
struct State {
    prefabs: tools::prefabs::Prefabs,
    selection: Vec<ObjectInfo>,
    active_tab: ActiveTab,
}

#[derive(Debug, Clone, Default)]
struct PrefabListItem {
    name: String,
    count: usize,
}

#[derive(Debug, Clone, Default)]
struct PrefabViewProps {
    active_tab: ActiveTab,
    selection_count: usize,
    prefabs: Vec<PrefabListItem>,
}

#[derive(Debug, Clone)]
enum PrefabViewMessage {
    ShowPrefabs,
    ShowTools,
    ReloadYaml,
    SavePrefab(String),
    SpawnPrefab(usize),
    PaintedTextInput(String),
    RotateInput(String),
    SplineDistribInput(String),
}

struct PrefabView;

impl ui::Component for PrefabView {
    type Props = PrefabViewProps;
    type Message = PrefabViewMessage;

    fn render(&self, props: Self::Props) -> ui::Node<Self::Message> {
        let tab_prefabs_style = if matches!(props.active_tab, ActiveTab::Prefabs) {
            BtnStyle::default().yellow().light().clickable()
        } else {
            BtnStyle::default().pale_blue().light().clickable()
        };
        let tab_tools_style = if matches!(props.active_tab, ActiveTab::Tools) {
            BtnStyle::default().yellow().light().clickable()
        } else {
            BtnStyle::default().pale_blue().light().clickable()
        };

        let panel = match props.active_tab {
            ActiveTab::Prefabs => ui::container()
                .flex()
                .flex_col()
                .w(48.)
                .with_child(
                    ui::container()
                        .flex()
                        .flex_row()
                        .w(48.)
                        .with_child(
                            ui::clickable(
                                "Reload YAML",
                                BtnStyle::default().pale_blue().light(),
                                PrefabViewMessage::ReloadYaml,
                            )
                            .w(24.)
                            .h(5.),
                        )
                        .with_child(
                            ui::typein(
                                "Save Selection",
                                BtnStyle::default().green().light(),
                                64,
                                PrefabViewMessage::SavePrefab,
                            )
                            .w(24.)
                            .h(5.),
                        ),
                )
                .with_children(props.prefabs.iter().enumerate().map(|(idx, prefab)| {
                    ui::clickable(
                        format!("{} [{}]", prefab.name, prefab.count),
                        BtnStyle::default().black().light().align_left(),
                        PrefabViewMessage::SpawnPrefab(idx),
                    )
                    .key(format!("prefab-{idx}"))
                    .w(48.)
                    .h(5.)
                })),
            ActiveTab::Tools => ui::container()
                .flex()
                .flex_col()
                .w(48.)
                .with_child(
                    ui::typein(
                        "Spline Distribution (m)",
                        BtnStyle::default().black().light(),
                        32,
                        PrefabViewMessage::SplineDistribInput,
                    )
                    .block()
                    .h(5.),
                )
                .with_child(
                    ui::typein(
                        "Paint Text",
                        BtnStyle::default().black().light(),
                        64,
                        PrefabViewMessage::PaintedTextInput,
                    )
                    .block()
                    .h(5.),
                )
                .with_child(
                    ui::typein(
                        "Rotate Selection (deg)",
                        BtnStyle::default().black().light(),
                        16,
                        PrefabViewMessage::RotateInput,
                    )
                    .block()
                    .h(5.),
                ),
        };

        ui::container()
            .flex()
            .flex_col()
            .w(175.)
            .items_end()
            .with_child(
                ui::container()
                    .mt(7.)
                    .flex()
                    .flex_row()
                    .w(48.)
                    .with_child(
                        ui::clickable("Prefabs", tab_prefabs_style, PrefabViewMessage::ShowPrefabs)
                            .h(5.)
                            .w(24.),
                    )
                    .with_child(
                        ui::clickable("Tools", tab_tools_style, PrefabViewMessage::ShowTools)
                            .h(5.)
                            .w(24.),
                    ),
            )
            .with_child(
                ui::text(
                    format!("Selection: {} object(s)", props.selection_count),
                    BtnStyle::default().dark().white(),
                )
                .w(48.)
                .h(5.),
            )
            .with_child(panel)
    }
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

    Ok(objects.len())
}

async fn handle_ui_message(
    connection: &mut FramedConnection,
    state: &mut State,
    msg: PrefabViewMessage,
) -> anyhow::Result<()> {
    match msg {
        PrefabViewMessage::ShowPrefabs => {
            state.active_tab = ActiveTab::Prefabs;
        },
        PrefabViewMessage::ShowTools => {
            state.active_tab = ActiveTab::Tools;
        },
        PrefabViewMessage::ReloadYaml => {
            let path = state.prefabs.path.clone();
            state.prefabs = tools::prefabs::Prefabs::load(path)?;
        },
        PrefabViewMessage::SavePrefab(name) => {
            let pending_name = name.trim().to_string();
            match state
                .prefabs
                .add_and_save_selection(&pending_name, &state.selection)
            {
                Ok(saved) => tracing::info!("Saved prefab '{saved}'"),
                Err(err) => tracing::warn!("save skipped: {err}"),
            }
        },
        PrefabViewMessage::SpawnPrefab(idx) => {
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
        PrefabViewMessage::PaintedTextInput(text) => {
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
        PrefabViewMessage::SplineDistribInput(input) => {
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
        PrefabViewMessage::RotateInput(input) => {
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
    }

    Ok(())
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
        active_tab: ActiveTab::Prefabs,
    };

    let view = PrefabView;
    let mut canvas = Canvas::<PrefabViewMessage>::new(ConnectionId::LOCAL);
    let mut blocked = false;
    let mut dirty = true;

    connection
        .write(TtcType::SelStart.with_request_id(REQI_SELECTION))
        .await?;

    loop {
        if dirty && !blocked {
            if let Some(diff) = canvas.reconcile(
                view.render(PrefabViewProps {
                    active_tab: state.active_tab,
                    selection_count: state.selection.len(),
                    prefabs: state
                        .prefabs
                        .data
                        .iter()
                        .map(|prefab| PrefabListItem {
                            name: prefab.name.clone(),
                            count: prefab.objects.len(),
                        })
                        .collect(),
                }),
            ) {
                for packet in diff.merge() {
                    connection.write(packet).await?;
                }
            }
            dirty = false;
        }

        tokio::select! {
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
                    Packet::Axm(axm) => {
                        if matches!(axm.pmoaction, PmoAction::TtcSel) && axm.reqi == REQI_SELECTION {
                            state.selection = axm.info;
                            dirty = true;
                            tracing::info!("Got Selection!");
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
