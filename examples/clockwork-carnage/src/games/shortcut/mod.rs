//! Shortcut mini-game subcommand. Players race from checkpoint1 to finish
//! and try to post the fastest time.
//!
//! Phase machine: Waiting → SettingUp → Racing (same pattern as bomb).

mod config;
mod events;
mod handlers;
mod state;
mod ui;

use std::time::Duration;

pub use config::{ShortcutArgs, ShortcutConfig, ShortcutRunConfig};
use handlers::{
    on_connected, on_disconnected, on_npl, on_race_ended, on_setup_aborted, on_setup_complete,
    on_toc, on_uco,
};
use kitcar::{App, AppError, Game, HandlerExt, Presence, Stage, State, run};
use state::{Shortcut, ShortcutGlobal, ShortcutPhase};
use tokio_util::sync::CancellationToken;
use ui::{ShortcutUi, ShortcutView};

pub async fn run_shortcut_with(cfg: ShortcutRunConfig) -> Result<(), AppError> {
    let app =
        App::<Shortcut>::with_state(Shortcut::new(cfg.config, CancellationToken::new(), cfg.db));
    app.state().write().runtime_cancel = app.cancel_token().clone();

    let sender = app.sender().clone();
    let ui = ShortcutUi::new(
        sender.clone(),
        ShortcutGlobal {
            phase: ShortcutPhase::Waiting.label().to_string(),
            ..Default::default()
        },
        |_ucid, _invalidator| ShortcutView,
    );

    let presence = Presence::new();
    let game = Game::new();

    let while_racing = |s: State<Shortcut>| s.read().phase == ShortcutPhase::Racing;

    let app = app
        .handle(Stage::Pre, presence)
        .handle(Stage::Pre, game)
        .handle(Stage::Pre, ui)
        .handle(Stage::Update, on_connected)
        .handle(Stage::Update, on_disconnected)
        .handle(Stage::Update, on_setup_complete)
        .handle(Stage::Update, on_setup_aborted)
        .handle(Stage::Update, on_race_ended)
        .handle(Stage::Update, on_npl.run_if(while_racing))
        .handle(Stage::Update, on_toc.run_if(while_racing))
        .handle(Stage::Update, on_uco.run_if(while_racing));

    let builder = insim::tcp(cfg.insim.addr)
        .isi_iname("shortcut".to_string())
        .isi_prefix('!')
        .isi_admin_password(cfg.insim.admin_password);

    run(builder, app).await
}

/// Ad-hoc entry point: runs shortcut without any DB backing.
pub async fn run_shortcut(args: ShortcutArgs) -> Result<(), AppError> {
    run_shortcut_with(ShortcutRunConfig {
        insim: args.insim,
        config: ShortcutConfig {
            track: args.track,
            layout: args.layout,
            setup_timeout: Duration::from_secs(60),
        },
        db: None,
    })
    .await
}
