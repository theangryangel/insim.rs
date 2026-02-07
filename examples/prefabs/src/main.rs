//! Prefab toolbox for LFS layout editing.
use std::{
    fs,
    net::SocketAddr,
    path::{Path, PathBuf},
};

use clap::{Parser, Subcommand};
use insim::{
    Packet, WithRequestId,
    core::{
        heading::Heading,
        object::{ObjectCoordinate, painted},
    },
    identifiers::{ConnectionId, RequestId},
    insim::{Axm, BfnType, BtnStyle, ObjectInfo, PmoAction, TtcType},
};
use kitcar::ui::{self, Canvas, Component};
use serde::{Deserialize, Serialize};

const REQI_SELECTION: RequestId = RequestId(200);

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Path to prefabs.yaml
    prefabs: PathBuf,
}

#[derive(Subcommand)]
enum Commands {
    /// Connect via UDP
    Udp {
        #[arg(long)]
        /// Local address to bind to. If not provided a random port will be used.
        bind: Option<SocketAddr>,

        #[arg(long)]
        /// host:port of LFS to connect to
        addr: SocketAddr,
    },

    /// Connect via TCP
    Tcp {
        #[arg(long)]
        /// host:port of LFS to connect to
        addr: SocketAddr,
    },
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
enum ActiveTab {
    #[default]
    Prefabs,
    Text,
    Tools,
}

#[derive(Debug, Clone)]
struct State {
    prefabs_path: PathBuf,
    prefabs: Vec<Prefab>,
    selection: Vec<ObjectInfo>,
    pending_name: String,
    awaiting_prefab_name: bool,
    pending_painted_text: String,
    awaiting_painted_text: bool,
    active_tab: ActiveTab,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PrefabFile {
    prefabs: Vec<Prefab>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Prefab {
    name: String,
    objects: Vec<ObjectInfo>,
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
    pending_name: String,
    awaiting_prefab_name: bool,
    pending_painted_text: String,
    awaiting_painted_text: bool,
}

#[derive(Debug, Clone)]
enum PrefabViewMessage {
    ShowPrefabs,
    ShowText,
    ShowTools,
    ReloadYaml,
    BeginSavePrefab,
    PrefabNameInput(String),
    SpawnPrefab(usize),
    BeginPaintText,
    PaintedTextInput(String),
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
        let tab_text_style = if matches!(props.active_tab, ActiveTab::Text) {
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
                            ui::clickable(
                                "Save Selection",
                                BtnStyle::default().green().light(),
                                PrefabViewMessage::BeginSavePrefab,
                            )
                            .w(24.)
                            .h(5.),
                        ),
                )
                .with_child_if(
                    ui::text(
                        if props.pending_name.is_empty() {
                            "new-prefab-name"
                        } else {
                            &props.pending_name
                        },
                        BtnStyle::default().white().dark(),
                    )
                    .typein(32, PrefabViewMessage::PrefabNameInput)
                    .w(48.)
                    .h(5.),
                    props.awaiting_prefab_name,
                )
                .with_children(props.prefabs.iter().enumerate().map(|(idx, prefab)| {
                    ui::clickable(
                        format!("{} [{}]", prefab.name, prefab.count),
                        BtnStyle::default().black().light().align_left(),
                        PrefabViewMessage::SpawnPrefab(idx),
                    )
                    .key(format!("prefab-{idx}"))
                    .w(48.)
                    .h(4.)
                })),
            ActiveTab::Text => ui::container()
                .flex()
                .flex_col()
                .w(48.)
                .with_child(
                    ui::clickable(
                        "Paint Text",
                        BtnStyle::default().yellow().light(),
                        PrefabViewMessage::BeginPaintText,
                    )
                    .w(48.)
                    .h(5.),
                )
                .with_child_if(
                    ui::text(
                        if props.pending_painted_text.is_empty() {
                            "type text and press enter"
                        } else {
                            &props.pending_painted_text
                        },
                        BtnStyle::default().white().dark(),
                    )
                    .typein(64, PrefabViewMessage::PaintedTextInput)
                    .w(48.)
                    .h(5.),
                    props.awaiting_painted_text,
                ),
            ActiveTab::Tools => ui::container().flex().flex_col().w(48.).with_child(
                ui::text(
                    "Tools palette reserved for future actions",
                    BtnStyle::default().black().light(),
                )
                .w(48.)
                .h(5.),
            ),
        };

        ui::container()
            .flex()
            .flex_col()
            .w(48.)
            .with_child(
                ui::text("Prefab Toolbox", BtnStyle::default().yellow().light())
                    .w(48.)
                    .h(5.),
            )
            .with_child(
                ui::container()
                    .flex()
                    .flex_row()
                    .w(48.)
                    .with_child(
                        ui::clickable("Prefabs", tab_prefabs_style, PrefabViewMessage::ShowPrefabs)
                            .w(16.)
                            .h(5.),
                    )
                    .with_child(
                        ui::clickable("Text", tab_text_style, PrefabViewMessage::ShowText)
                            .w(16.)
                            .h(5.),
                    )
                    .with_child(
                        ui::clickable("Tools", tab_tools_style, PrefabViewMessage::ShowTools)
                            .w(16.)
                            .h(5.),
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

fn load_prefabs(path: &Path) -> Result<Vec<Prefab>, String> {
    if !path.exists() {
        return Ok(Vec::new());
    }

    let raw = fs::read_to_string(path)
        .map_err(|e| format!("failed to read '{}': {e}", path.display()))?;
    if raw.trim().is_empty() {
        return Ok(Vec::new());
    }

    let data: PrefabFile = serde_yaml::from_str(&raw)
        .map_err(|e| format!("failed to parse '{}': {e}", path.display()))?;

    Ok(data.prefabs)
}

fn save_prefabs(path: &Path, prefabs: &[Prefab]) -> Result<(), String> {
    let data = PrefabFile {
        prefabs: prefabs.to_vec(),
    };
    let yaml = serde_yaml::to_string(&data)
        .map_err(|e| format!("failed to serialize prefab yaml: {e}"))?;
    fs::write(path, yaml).map_err(|e| format!("failed to write '{}': {e}", path.display()))
}

fn build_props(state: &State) -> PrefabViewProps {
    PrefabViewProps {
        active_tab: state.active_tab,
        selection_count: state.selection.len(),
        prefabs: state
            .prefabs
            .iter()
            .map(|prefab| PrefabListItem {
                name: prefab.name.clone(),
                count: prefab.objects.len(),
            })
            .collect(),
        pending_name: state.pending_name.clone(),
        awaiting_prefab_name: state.awaiting_prefab_name,
        pending_painted_text: state.pending_painted_text.clone(),
        awaiting_painted_text: state.awaiting_painted_text,
    }
}

async fn render_ui(
    connection: &mut FramedConnection,
    canvas: &mut Canvas<PrefabViewMessage>,
    view: &PrefabView,
    state: &State,
) -> insim::Result<()> {
    if let Some(diff) = canvas.reconcile(view.render(build_props(state))) {
        for packet in diff.merge() {
            connection.write(packet).await?;
        }
    }

    Ok(())
}

fn unique_prefab_name(existing: &[Prefab], requested: &str) -> String {
    let trimmed = requested.trim();
    let base = if trimmed.is_empty() {
        "prefab".to_string()
    } else {
        trimmed.to_string()
    };

    if !existing.iter().any(|p| p.name == base) {
        return base;
    }

    let mut n = 2_u32;
    loop {
        let candidate = format!("{}-{n}", base);
        if !existing.iter().any(|p| p.name == candidate) {
            return candidate;
        }
        n = n.saturating_add(1);
    }
}

fn to_relative(selection: &[ObjectInfo]) -> Vec<ObjectInfo> {
    if selection.is_empty() {
        return Vec::new();
    }

    let anchor = *selection[0].position();
    selection
        .iter()
        .cloned()
        .map(|mut obj| {
            let pos = obj.position_mut();
            pos.x = clamp_i16(i32::from(pos.x) - i32::from(anchor.x));
            pos.y = clamp_i16(i32::from(pos.y) - i32::from(anchor.y));
            pos.z = clamp_u8(i32::from(pos.z) - i32::from(anchor.z));
            obj
        })
        .collect()
}

fn place_at_anchor(objects: &[ObjectInfo], anchor: ObjectCoordinate) -> Vec<ObjectInfo> {
    objects
        .iter()
        .cloned()
        .map(|mut obj| {
            let pos = obj.position_mut();
            pos.x = clamp_i16(i32::from(anchor.x) + i32::from(pos.x));
            pos.y = clamp_i16(i32::from(anchor.y) + i32::from(pos.y));
            pos.z = clamp_u8(i32::from(anchor.z) + i32::from(pos.z));
            obj
        })
        .collect()
}

fn painted_letters_from_text(
    text: &str,
    anchor: ObjectCoordinate,
    heading: Heading,
) -> Vec<ObjectInfo> {
    const SPACING_RAW_UNITS: i32 = 16;
    const PADDING_SLOTS: i32 = 2;

    let radians = heading.to_radians();
    let right_x = radians.cos();
    let right_y = radians.sin();
    let anchor_x = f64::from(anchor.x);
    let anchor_y = f64::from(anchor.y);

    text.chars()
        .enumerate()
        .filter_map(|(index, ch)| {
            let character = painted::Character::try_from(ch).ok()?;
            let slot = i32::try_from(index).ok()?.saturating_add(PADDING_SLOTS);
            let offset = f64::from(slot.saturating_mul(SPACING_RAW_UNITS));

            let x = clamp_i16((anchor_x + (right_x * offset)).round() as i32);
            let y = clamp_i16((anchor_y + (right_y * offset)).round() as i32);

            Some(ObjectInfo::PaintLetters(painted::Letters {
                xyz: ObjectCoordinate { x, y, z: anchor.z },
                colour: painted::PaintColour::Yellow,
                character,
                heading,
                floating: false,
            }))
        })
        .collect()
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
) -> insim::Result<usize> {
    let count = objects.len();
    if count == 0 {
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

    connection
        .write(Axm {
            ucid: ConnectionId::LOCAL,
            pmoaction: PmoAction::Selection,
            info: objects,
            ..Default::default()
        })
        .await?;

    Ok(count)
}

fn save_current_selection(state: &mut State) -> Result<String, String> {
    if state.selection.is_empty() {
        return Err("cannot save prefab: selection is empty".to_string());
    }

    let name = unique_prefab_name(&state.prefabs, &state.pending_name);
    let relative = to_relative(&state.selection);
    state.prefabs.push(Prefab {
        name: name.clone(),
        objects: relative,
    });
    save_prefabs(&state.prefabs_path, &state.prefabs)?;
    state.pending_name = name.clone();

    Ok(name)
}

async fn handle_ui_message(
    connection: &mut FramedConnection,
    state: &mut State,
    msg: PrefabViewMessage,
) -> insim::Result<()> {
    match msg {
        PrefabViewMessage::ShowPrefabs => {
            state.active_tab = ActiveTab::Prefabs;
        },
        PrefabViewMessage::ShowText => {
            state.active_tab = ActiveTab::Text;
        },
        PrefabViewMessage::ShowTools => {
            state.active_tab = ActiveTab::Tools;
        },
        PrefabViewMessage::ReloadYaml => match load_prefabs(&state.prefabs_path) {
            Ok(prefabs) => {
                state.prefabs = prefabs;
                tracing::info!("Reloaded prefabs from disk");
            },
            Err(err) => tracing::error!("reload failed: {err}"),
        },
        PrefabViewMessage::BeginSavePrefab => {
            state.awaiting_prefab_name = true;
            state.awaiting_painted_text = false;
            state.active_tab = ActiveTab::Prefabs;
        },
        PrefabViewMessage::PrefabNameInput(name) => {
            state.pending_name = name.trim().to_string();
            if state.awaiting_prefab_name {
                match save_current_selection(state) {
                    Ok(saved) => tracing::info!("Saved prefab '{saved}'"),
                    Err(err) => tracing::warn!("save skipped: {err}"),
                }
                state.awaiting_prefab_name = false;
            }
        },
        PrefabViewMessage::SpawnPrefab(idx) => {
            if let Some(prefab) = state.prefabs.get(idx) {
                let anchor = state
                    .selection
                    .first()
                    .map(|obj| *obj.position())
                    .unwrap_or_default();
                let placed = place_at_anchor(&prefab.objects, anchor);
                let _ = spawn_at_selection(connection, state, placed).await?;
            }
        },
        PrefabViewMessage::BeginPaintText => {
            state.awaiting_painted_text = true;
            state.awaiting_prefab_name = false;
            state.active_tab = ActiveTab::Text;
        },
        PrefabViewMessage::PaintedTextInput(text) => {
            let text = text.trim().to_string();
            state.pending_painted_text = text.clone();
            state.awaiting_painted_text = false;

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
                let painted_text = painted_letters_from_text(&text, anchor, heading);
                let painted_count = spawn_at_selection(connection, state, painted_text).await?;

                if painted_count == 0 {
                    tracing::warn!(
                        "paint skipped: text has no supported painted-letter characters"
                    );
                } else {
                    tracing::info!("Painted {painted_count} letter objects into selection");
                }
            }
        },
    }

    Ok(())
}

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_tracing_subscriber();

    let cli = Cli::parse();
    let prefabs = load_prefabs(&cli.prefabs).map_err(std::io::Error::other)?;

    let mut builder = match &cli.command {
        Commands::Udp { bind, addr } => {
            tracing::info!("Connecting via UDP to {addr}");
            insim::udp(*addr, *bind)
        },
        Commands::Tcp { addr } => {
            tracing::info!("Connecting via TCP to {addr}");
            insim::tcp(*addr)
        },
    };
    builder = builder
        .isi_iname(Some("prefab-toolbox".to_string()))
        .isi_flag_local(true)
        .isi_flag_axm_edit(true);

    let mut connection = builder.connect_async().await?;
    tracing::info!("Connected");

    let mut state = State {
        prefabs_path: cli.prefabs,
        prefabs,
        selection: Vec::new(),
        pending_name: String::new(),
        awaiting_prefab_name: false,
        pending_painted_text: String::new(),
        awaiting_painted_text: false,
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
            render_ui(&mut connection, &mut canvas, &view, &state).await?;
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
                        }
                    }
                    Packet::Btc(btc) => {
                        if let Some(msg) = canvas.translate_clickid(&btc.clickid) {
                            handle_ui_message(&mut connection, &mut state, msg).await?;
                            dirty = true;
                        }
                    }
                    Packet::Btt(btt) => {
                        if let Some(msg) = canvas.translate_typein_clickid(&btt.clickid, btt.text) {
                            handle_ui_message(&mut connection, &mut state, msg).await?;
                            dirty = true;
                        }
                    }
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
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(())
}
