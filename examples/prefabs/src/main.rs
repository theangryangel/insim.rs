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
    identifiers::{ClickId, ConnectionId, RequestId},
    insim::{Axm, Bfn, BfnType, Btn, BtnStyle, ObjectInfo, PmoAction, TtcType},
};
use serde::{Deserialize, Serialize};

const REQI_SELECTION: RequestId = RequestId(200);

const BTN_TITLE: ClickId = ClickId(10);
const BTN_RELOAD: ClickId = ClickId(11);
const BTN_SAVE: ClickId = ClickId(12);
const BTN_NAME_INPUT: ClickId = ClickId(13);
const BTN_PAINT_TEXT: ClickId = ClickId(14);
const BTN_PAINT_TEXT_INPUT: ClickId = ClickId(15);
const BTN_PREFAB_FIRST: u8 = 20;
const BTN_PREFAB_MAX: u8 = ClickId::MAX;

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

#[derive(Debug, Clone)]
struct State {
    prefabs_path: PathBuf,
    prefabs: Vec<Prefab>,
    selection: Vec<ObjectInfo>,
    pending_name: String,
    awaiting_prefab_name: bool,
    pending_painted_text: String,
    awaiting_painted_text: bool,
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

async fn draw_ui(
    connection: &mut insim::net::tokio_impl::Framed,
    state: &State,
) -> insim::Result<()> {
    connection
        .write(Bfn {
            subt: BfnType::DelBtn,
            clickid: BTN_TITLE,
            clickmax: BTN_PREFAB_MAX,
            ucid: ConnectionId::LOCAL,
            ..Default::default()
        })
        .await?;

    connection
        .write(Btn {
            clickid: BTN_TITLE,
            reqi: RequestId(BTN_TITLE.0),
            ucid: ConnectionId::LOCAL,
            l: 2,
            t: 20,
            w: 45,
            h: 5,
            text: "Prefab Toolbox".to_string(),
            bstyle: BtnStyle::default().yellow().light(),
            ..Default::default()
        })
        .await?;

    connection
        .write(Btn {
            clickid: BTN_RELOAD,
            reqi: RequestId(BTN_RELOAD.0),
            ucid: ConnectionId::LOCAL,
            l: 2,
            t: 26,
            w: 22,
            h: 5,
            text: "Reload YAML".to_string(),
            bstyle: BtnStyle::default().pale_blue().light().clickable(),
            ..Default::default()
        })
        .await?;

    connection
        .write(Btn {
            clickid: BTN_SAVE,
            reqi: RequestId(BTN_SAVE.0),
            ucid: ConnectionId::LOCAL,
            l: 25,
            t: 26,
            w: 22,
            h: 5,
            text: "Save Selection".to_string(),
            bstyle: BtnStyle::default().green().light().clickable(),
            ..Default::default()
        })
        .await?;

    let mut next_row: u8 = 32;

    if state.awaiting_prefab_name {
        connection
            .write(Btn {
                clickid: BTN_NAME_INPUT,
                reqi: RequestId(BTN_NAME_INPUT.0),
                ucid: ConnectionId::LOCAL,
                l: 2,
                t: next_row,
                w: 45,
                h: 5,
                typein: Some(32),
                text: if state.pending_name.is_empty() {
                    "new-prefab-name".to_string()
                } else {
                    state.pending_name.clone()
                },
                bstyle: BtnStyle::default().white().dark().clickable(),
                ..Default::default()
            })
            .await?;
        next_row = next_row.saturating_add(6);
    }

    connection
        .write(Btn {
            clickid: BTN_PAINT_TEXT,
            reqi: RequestId(BTN_PAINT_TEXT.0),
            ucid: ConnectionId::LOCAL,
            l: 2,
            t: next_row,
            w: 45,
            h: 5,
            text: "Paint Text".to_string(),
            bstyle: BtnStyle::default().yellow().light().clickable(),
            ..Default::default()
        })
        .await?;
    next_row = next_row.saturating_add(6);

    if state.awaiting_painted_text {
        connection
            .write(Btn {
                clickid: BTN_PAINT_TEXT_INPUT,
                reqi: RequestId(BTN_PAINT_TEXT_INPUT.0),
                ucid: ConnectionId::LOCAL,
                l: 2,
                t: next_row,
                w: 45,
                h: 5,
                typein: Some(64),
                text: if state.pending_painted_text.is_empty() {
                    "type text and press enter".to_string()
                } else {
                    state.pending_painted_text.clone()
                },
                bstyle: BtnStyle::default().white().dark().clickable(),
                ..Default::default()
            })
            .await?;
        next_row = next_row.saturating_add(6);
    }

    let mut click = BTN_PREFAB_FIRST;
    let mut y = next_row;
    for prefab in &state.prefabs {
        if click > BTN_PREFAB_MAX {
            break;
        }

        connection
            .write(Btn {
                clickid: ClickId(click),
                reqi: RequestId(click),
                ucid: ConnectionId::LOCAL,
                l: 2,
                t: y,
                w: 45,
                h: 4,
                text: format!("{} [{}]", prefab.name, prefab.objects.len()),
                bstyle: BtnStyle::default().black().light().clickable().align_left(),
                ..Default::default()
            })
            .await?;

        click = click.saturating_add(1);
        y = y.saturating_add(4);
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
    connection: &mut insim::net::tokio_impl::Framed,
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
    };

    draw_ui(&mut connection, &state).await?;

    connection
        .write(TtcType::SelStart.with_request_id(REQI_SELECTION))
        .await?;

    loop {
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
                        }
                    }
                    Packet::Btt(btt) => {
                        if btt.clickid == BTN_NAME_INPUT {
                            state.pending_name = btt.text.trim().to_string();

                            if state.awaiting_prefab_name {
                                match save_current_selection(&mut state) {
                                    Ok(name) => tracing::info!("Saved prefab '{name}'"),
                                    Err(err) => tracing::warn!("save skipped: {err}"),
                                }
                                state.awaiting_prefab_name = false;
                            }

                            draw_ui(&mut connection, &state).await?;
                        } else if btt.clickid == BTN_PAINT_TEXT_INPUT {
                            let text = btt.text.trim().to_string();
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
                                let painted_count =
                                    spawn_at_selection(&mut connection, &state, painted_text).await?;
                                if painted_count == 0 {
                                    tracing::warn!(
                                        "paint skipped: text has no supported painted-letter characters"
                                    );
                                } else {
                                    tracing::info!(
                                        "Painted {painted_count} letter objects into selection"
                                    );
                                }
                            }

                            draw_ui(&mut connection, &state).await?;
                        }
                    }
                    Packet::Bfn(bfn) => {
                        if matches!(bfn.subt, BfnType::UserClear | BfnType::BtnRequest) {
                            draw_ui(&mut connection, &state).await?;
                        }
                    }
                    Packet::Btc(btc) => {
                        if btc.clickid == BTN_RELOAD {
                            match load_prefabs(&state.prefabs_path) {
                                Ok(prefabs) => {
                                    state.prefabs = prefabs;
                                    tracing::info!("Reloaded prefabs from disk");
                                }
                                Err(err) => tracing::error!("reload failed: {err}"),
                            }
                            draw_ui(&mut connection, &state).await?;
                            continue;
                        }

                        if btc.clickid == BTN_SAVE {
                            state.awaiting_prefab_name = true;
                            state.awaiting_painted_text = false;
                            draw_ui(&mut connection, &state).await?;
                            continue;
                        }

                        if btc.clickid == BTN_PAINT_TEXT {
                            state.awaiting_painted_text = true;
                            state.awaiting_prefab_name = false;
                            draw_ui(&mut connection, &state).await?;
                            continue;
                        }

                        if btc.clickid.0 >= BTN_PREFAB_FIRST {
                            let idx = usize::from(btc.clickid.0 - BTN_PREFAB_FIRST);
                            if let Some(prefab) = state.prefabs.get(idx) {
                                let anchor = state
                                    .selection
                                    .first()
                                    .map(|obj| *obj.position())
                                    .unwrap_or_default();
                                let placed = place_at_anchor(&prefab.objects, anchor);
                                spawn_at_selection(&mut connection, &state, placed).await?;
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(())
}
