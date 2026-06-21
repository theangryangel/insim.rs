//! Metronome mini-game subcommand. Players drive from checkpoint1 to finish
//! and try to match the target duration as closely as possible.
//!
//! Round lifecycle is owned by [`kitcar::RoundManager`].

mod config;
mod handlers;
mod state;
mod ui;

use std::time::Duration;

use config::MIN_PLAYERS;
pub use config::{MetronomeArgs, MetronomeConfig, MetronomeRunConfig};
use handlers::{
    on_connected, on_disconnected, on_player_left, on_player_teleported_to_pits, on_round_ended,
    on_round_started, on_toc, on_uco,
};
use insim::insim::RaceLaps;
use kitcar::{App, AppError, HandlerExt, RoundManager, RoundPolicy, RoundSpec, Stage, run};
use state::{Metronome, MetronomeGlobal};
use ui::{MetronomeUi, MetronomeView};

pub async fn run_metronome_with(cfg: MetronomeRunConfig) -> Result<(), AppError> {
    let target = cfg.config.target;
    let rounds = RoundManager::new(
        RoundPolicy {
            min_players: MIN_PLAYERS,
            setup_timeout: cfg.config.setup_timeout,
        },
        vec![RoundSpec {
            track: cfg.config.track,
            laps: RaceLaps::Untimed,
            wind: 0,
            layout: cfg.config.layout.clone(),
        }],
    );

    let app = App::<Metronome>::with_state(Metronome::new(cfg.config, cfg.db));
    let sender = app.sender().clone();
    let ui = MetronomeUi::new(
        sender.clone(),
        MetronomeGlobal {
            target,
            ..Default::default()
        },
        |_ucid, _invalidator| MetronomeView,
    );

    let while_racing = |r: RoundManager| r.is_racing();

    let app = app
        .with_ui(ui)
        .handle(Stage::Pre, rounds)
        .handle(Stage::Update, on_connected)
        .handle(Stage::Update, on_disconnected)
        .handle(Stage::Update, on_round_started)
        .handle(Stage::Update, on_round_ended)
        .handle(Stage::Update, on_player_left.run_if(while_racing))
        .handle(
            Stage::Update,
            on_player_teleported_to_pits.run_if(while_racing),
        )
        .handle(Stage::Update, on_toc.run_if(while_racing))
        .handle(Stage::Update, on_uco.run_if(while_racing));

    let builder = insim::tcp(cfg.insim.addr)
        .isi_iname("metronome".to_string())
        .isi_prefix('!')
        .isi_admin_password(cfg.insim.admin_password);

    run(builder, app).await
}

/// Ad-hoc entry point: runs metronome without any DB backing.
pub async fn run_metronome(args: MetronomeArgs) -> Result<(), AppError> {
    run_metronome_with(MetronomeRunConfig {
        insim: args.insim,
        config: MetronomeConfig {
            target: Duration::from_millis(args.target_ms),
            track: args.track,
            layout: args.layout,
            setup_timeout: Duration::from_secs(60),
        },
        db: None,
    })
    .await
}
