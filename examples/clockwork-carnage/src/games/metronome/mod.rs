//! Metronome mini-game subcommand. Players drive from checkpoint1 to finish
//! and try to match the target duration as closely as possible.
//!
//! Phase machine: Waiting → SettingUp → Racing (same pattern as bomb).

mod config;
mod events;
mod handlers;
mod state;
mod ui;

use std::time::Duration;

pub use config::{MetronomeArgs, MetronomeConfig, MetronomeRunConfig};
use handlers::{
    on_connected, on_disconnected, on_race_ended, on_setup_aborted, on_setup_complete, on_toc,
    on_uco,
};
use kitcar::{App, AppError, Game, HandlerExt, Presence, Stage, State, run};
use state::{Metronome, MetronomeGlobal, MetronomePhase};
use tokio_util::sync::CancellationToken;
use ui::{MetronomeUi, MetronomeView};

pub async fn run_metronome_with(cfg: MetronomeRunConfig) -> Result<(), AppError> {
    let target = cfg.config.target;
    let app =
        App::<Metronome>::with_state(Metronome::new(cfg.config, CancellationToken::new(), cfg.db));
    app.state().write().runtime_cancel = app.cancel_token().clone();

    let sender = app.sender().clone();
    let ui = MetronomeUi::new(
        sender.clone(),
        MetronomeGlobal {
            target,
            ..Default::default()
        },
        |_ucid, _invalidator| MetronomeView,
    );

    let presence = Presence::new();
    let game = Game::new();

    let while_racing = |s: State<Metronome>| s.read().phase == MetronomePhase::Racing;

    let app = app
        .handle(Stage::Pre, presence)
        .handle(Stage::Pre, game)
        .handle(Stage::Pre, ui)
        .handle(Stage::Update, on_connected)
        .handle(Stage::Update, on_disconnected)
        .handle(Stage::Update, on_setup_complete)
        .handle(Stage::Update, on_setup_aborted)
        .handle(Stage::Update, on_race_ended)
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
