//! Clockwork Carnage — unified binary (InSim runner + web dashboard).

#![allow(missing_docs, missing_debug_implementations)]

mod db;

mod games;
mod hud;
mod web;

type ChatError = kitcar::chat::RuntimeError;
const MIN_PLAYERS: usize = 2;

use std::net::SocketAddr;

use anyhow::Context as _;
use clap::Parser;
use games::MiniGameCtx;
use insim::{WithRequestId, insim::TinyType};
use kitcar::{game, presence};

// -- Config -------------------------------------------------------------------

#[derive(Debug, serde::Deserialize)]
struct Config {
    #[serde(default = "default_db")]
    db: String,
    insim: Option<InsimSection>,
    web: Option<WebSection>,
}

#[derive(Debug, serde::Deserialize)]
struct InsimSection {
    addr: SocketAddr,
    password: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
struct WebSection {
    #[serde(default = "default_listen")]
    listen: SocketAddr,
    base_url: String,
    oauth_client_id: String,
    oauth_client_secret: String,
    session_key: Option<String>,
}

fn default_db() -> String {
    "clockwork-carnage.db".to_owned()
}

fn default_listen() -> SocketAddr {
    "127.0.0.1:3000".parse().unwrap()
}

fn load_config(path: &std::path::Path) -> anyhow::Result<Config> {
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("cannot read config file {path:?}"))?;
    let config: Config =
        toml::from_str(&text).with_context(|| format!("cannot parse config file {path:?}"))?;
    if config.insim.is_none() && config.web.is_none() {
        anyhow::bail!("at least one of [insim] or [web] must be present in config");
    }
    if let Some(web) = &config.web {
        let key_len = web.session_key.as_deref().unwrap_or("").len();
        if key_len > 0 && key_len < 64 {
            anyhow::bail!("`session_key` must be at least 64 bytes (got {key_len})");
        }
    }
    Ok(config)
}

// -- CLI ----------------------------------------------------------------------

#[derive(Debug, Parser)]
struct Args {
    #[arg(long, default_value = "clockwork-carnage.toml")]
    config: std::path::PathBuf,
}

use tokio::task::JoinSet;

// -- Runner -------------------------------------------------------------------

async fn run_loop(pool: db::Pool, config: Config) -> anyhow::Result<()> {
    let mut set = JoinSet::new();
    let base_url = config.web.as_ref().map(|w| w.base_url.clone());

    // InSim setup — only if [insim] present
    let (ctx, web_presence) = if let Some(insim_cfg) = config.insim {
        let (insim, insim_handle) = insim::tcp(insim_cfg.addr)
            .isi_admin_password(insim_cfg.password)
            .isi_iname("carnage".to_owned())
            .isi_prefix('!')
            .isi_flag_mso_cols(true)
            .isi_flag_mci(true)
            .isi_interval(std::time::Duration::from_millis(250))
            .spawn(100)
            .await?;

        tracing::info!("Connected to InSim");

        let (presence, presence_handle) = presence::spawn(insim.clone(), 32);
        let (game, game_handle) = game::spawn(insim.clone(), 32);
        let user_sync_handle = db::spawn_user_sync(&presence, pool.clone());

        insim.send(TinyType::Ncn.with_request_id(1)).await?;
        insim.send(TinyType::Npl.with_request_id(2)).await?;
        insim.send(TinyType::Sst.with_request_id(3)).await?;

        for &cmd in &["/select no", "/vote no", "/autokick no"] {
            insim.send_command(cmd).await?;
        }

        let _ = set.spawn(async move {
            insim_handle
                .await
                .context("InSim background task panicked")?
                .context("InSim background task failed")
        });

        let _ = set.spawn(async move {
            presence_handle
                .await
                .context("Presence background task panicked")?
                .map_err(|e| anyhow::anyhow!("Presence background task failed: {e}"))
        });

        let _ = set.spawn(async move {
            game_handle
                .await
                .context("Game background task panicked")?
                .map_err(|e| anyhow::anyhow!("Game background task failed: {e}"))
        });

        let _ = set.spawn(async move {
            user_sync_handle
                .await
                .context("User sync background task panicked")?
                .map_err(|e| anyhow::anyhow!("User sync background task failed: {e}"))
        });

        let ctx = MiniGameCtx {
            pool: pool.clone(),
            insim: insim.clone(),
            presence: presence.clone(),
            game: game.clone(),
            base_url: base_url.clone(),
        };

        (Some(ctx), Some(presence))
    } else {
        (None, None)
    };

    // Orchestrator (Scheduler + Game Manager + Announcer)
    let _ = set.spawn({
        let pool = pool.clone();
        async move { games::MiniGameManager::new(pool, ctx).run().await }
    });

    // Web — only if [web] present
    if let Some(w) = config.web {
        let web_cfg = web::WebConfig {
            base_url: w.base_url,
            oauth_client_id: w.oauth_client_id,
            oauth_client_secret: w.oauth_client_secret,
            session_key: w.session_key.unwrap_or_else(|| "a".repeat(64)),
        };

        let _ = set.spawn({
            let pool = pool.clone();
            let web_presence = web_presence.clone();
            async move { web::serve(w.listen, pool, web_cfg, web_presence).await }
        });
    }

    if let Some(res) = set.join_next().await {
        match res {
            Ok(Ok(())) => tracing::info!("A background task exited naturally"),
            Ok(Err(e)) => tracing::error!("A background task failed: {e:?}"),
            Err(e) => tracing::error!("A background task panicked: {e}"),
        }
    }

    Ok(())
}

// -- Entry point --------------------------------------------------------------

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive(tracing_subscriber::filter::LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .init();

    let args = Args::parse();
    let config = load_config(&args.config)?;
    let pool = db::connect(&config.db).await?;

    run_loop(pool, config).await?;

    Ok(())
}
