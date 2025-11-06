//! 20s league
mod chat;
mod combo;
mod components;
mod config;
mod db;
mod scenes;
mod context;
mod cli;

use std::{sync::Arc, time::Duration};

use clap::{Parser};
use anyhow::Result;
use insim::{WithRequestId, identifiers::ConnectionId, insim::TinyType};
use kitcar::{
    chat::Parse,
    game::GameInfo,
    leaderboard::Leaderboard,
    presence::Presence,
    ui,
};
use tokio::{task::JoinHandle, time::timeout};
use tokio_util::sync::CancellationToken;

use crate::{context::Context, chat::Chat, components::RootProps, scenes::Scene};

async fn run(repo: db::Repo, addr: &str, admin: Option<String>) -> anyhow::Result<()> {
    let config = Arc::new(config::Config::from_file("config.yaml")?);

    tracing::info!("{:?}", config);

    let (insim, _join_handle) = insim::tcp(addr)
        .isi_admin_password(admin)
        .isi_iname("cadence-cup".to_owned())
        .isi_prefix('!')
        .spawn(100)
        .await?;

    let (ui_handle, _ui_thread) = ui::Manager::spawn::<components::Root>(
        insim.clone(),
        RootProps {
            scene: components::RootScene::Idle,
        },
    );

    tracing::info!("20 Second League started!");

    let mut packets = insim.subscribe();
    let mut scene_handle: Option<JoinHandle<anyhow::Result<Option<Scene>>>> = None;

    // FIXME - scene recovery from database?
    let mut scene: Scene = scenes::Idle.into();

    let cx = Context {
        insim: insim.clone(),
        ui: ui_handle.clone(),
        presence: Presence::spawn(insim.clone(), 32),
        game: GameInfo::spawn(insim.clone(), 32),
        // FIXME: with database this is unrequired probably
        leaderboard: Leaderboard::<String>::spawn(32),
        config: config.clone(),
        shutdown: CancellationToken::new(),
        database: repo,
    };

    insim.send(TinyType::Ncn.with_request_id(1)).await?;
    insim.send(TinyType::Npl.with_request_id(2)).await?;
    insim.send(TinyType::Sst.with_request_id(3)).await?;

    loop {
        // get a temporary handle for the select loop below
        // FIXME: see above note about consolidating this into a fn on Scene
        let handle = scene_handle.get_or_insert_with(|| scene.clone().spawn(cx.clone()));

        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                cx.shutdown.cancel();
                // we can take this because we're shutting down
                if let Some(scene_handle) = scene_handle {
                    tracing::info!("Waiting 5 seconds for graceful shutdown...");
                    let _ = timeout(Duration::from_secs(5), scene_handle).await?;
                }
                break;

            },
            result = handle => {
                scene_handle = None;
                match result?? {
                    Some(next) => { scene = next; },
                    None => { break; },
                }
            },
            packet = packets.recv() => {
                match packet? {
                    insim::Packet::Ncn(ncn) if ncn.ucid != ConnectionId::LOCAL => {
                        cx.database.upsert_player(&ncn.uname, &ncn.pname)?;
                    },
                    insim::Packet::Mso(mso) => {
                        match Chat::parse(mso.msg_from_textstart()) {
                            Ok(Chat::Quit) => {
                                if_chain::if_chain! {
                                    if let Some(conn_info) = cx.presence.connection(&mso.ucid).await;
                                    if conn_info.admin;
                                    then {
                                        insim.send_message("Quitting.. bye!", ConnectionId::ALL).await?;
                                        break;
                                    }
                                }
                            },
                            Ok(Chat::Echo { message }) => {
                                insim.send_message(&format!("Echo: {}", message), mso.ucid).await?;
                            }
                            Ok(Chat::Help) => {
                                insim.send_message("Available commands:", mso.ucid).await?;
                                for cmd in Chat::help() {
                                    insim.send_message(cmd, mso.ucid).await?;
                                }
                            },
                            _ => {},
                        }
                    },
                    _ => {}
                }
            }
        }
    }

    Ok(())

}

#[tokio::main]
async fn main() -> Result<()> {
    // Setup with a default log level of INFO RUST_LOG is unset
    tracing_subscriber::fmt::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive(tracing_subscriber::filter::LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .init();

    let cli = cli::Args::parse();

    let repo = db::Repo::new(&cli.database);
    repo.migrate()?;

    println!("{:?}", cli);

    match cli.cmd {
        cli::Cmd::Combo(cli::ComboCmd::Add { .. }) => todo!(),
        cli::Cmd::Combo(cli::ComboCmd::List) => todo!(),
        cli::Cmd::Combo(cli::ComboCmd::Delete) => todo!(),
        cli::Cmd::Run { addr, admin, lobby_duration, victory_duration, max_scoring_players } => run(repo, &addr, admin).await
    }
}
