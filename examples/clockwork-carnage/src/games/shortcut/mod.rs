//! Shortcut mini-game subcommand. Players race from checkpoint1 to finish
//! and try to post the fastest time.
//!
//! Round lifecycle is owned by [`kitcar::RoundManager`].

mod config;
mod handlers;
mod state;
mod ui;

use std::time::Duration;

use config::MIN_PLAYERS;
pub use config::{ShortcutArgs, ShortcutConfig, ShortcutRunConfig};
use handlers::{
    on_connected, on_disconnected, on_player_left, on_player_teleported_to_pits, on_round_ended,
    on_round_started, on_toc, on_uco,
};
use insim::insim::RaceLaps;
use kitcar::{
    App, AppError, HandlerExt, RoundManager, RoundPhase, RoundPolicy, RoundSpec, Stage, run,
};
use state::{Shortcut, ShortcutGlobal};
use ui::{ShortcutUi, ShortcutView};

pub async fn run_shortcut_with(cfg: ShortcutRunConfig) -> Result<(), AppError> {
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

    let app = App::<Shortcut>::with_state(Shortcut::new(cfg.db));
    let sender = app.sender().clone();
    let ui = ShortcutUi::new(
        sender.clone(),
        ShortcutGlobal {
            phase: RoundPhase::Waiting.to_string(),
            ..Default::default()
        },
        |_ucid, _invalidator| ShortcutView,
    );

    let while_racing = |r: RoundManager| r.is_racing();

    let app = app
        .handle(Stage::Pre, ui)
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
