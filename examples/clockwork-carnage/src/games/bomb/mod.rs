//! Bomb mini-game subcommand. Ported from kitcar/examples/bomb.rs with
//! optional DB writes when `--db` and `--event-id` are provided.

mod config;
mod db;
mod events;
mod handlers;
mod state;
mod ui;

use std::time::Duration;

pub use config::{BombArgs, BombConfig, BombRunConfig};
use config::{PENALTY_CLEAR_DELAY, TICK_PERIOD};
use events::BombTick;
use handlers::{
    on_con, on_connected, on_crs, on_disconnected, on_npl, on_pit, on_player_left,
    on_player_teleported_to_pits, on_race_ended, on_setup_aborted, on_setup_complete, on_tick,
    on_toc, on_uco,
};
use kitcar::{App, AppError, Game, HandlerExt, PenaltyClearer, Presence, Stage, State, run};
use state::{Bomb, BombGlobal, BombPhase};
use tokio_util::sync::CancellationToken;
use ui::{BombUi, BombView};

pub async fn run_bomb_with(cfg: BombRunConfig) -> Result<(), AppError> {
    let app = App::<Bomb>::with_state(Bomb::new(cfg.config, CancellationToken::new(), cfg.db));
    let sender = app.sender().clone();

    app.state().write().runtime_cancel = app.cancel_token().clone();

    let ui = BombUi::new(
        sender.clone(),
        BombGlobal {
            phase: BombPhase::Waiting.label().to_string(),
            ..Default::default()
        },
        |_ucid, invalidator| {
            let _tick_handle = tokio::spawn(async move {
                let mut interval = tokio::time::interval(Duration::from_millis(100));
                loop {
                    let _ = interval.tick().await;
                    invalidator.invalidate();
                }
            });
            BombView { _tick_handle }
        },
    );

    let presence = Presence::new();
    let game = Game::new();
    let clearer = PenaltyClearer::new(PENALTY_CLEAR_DELAY);

    let while_racing = |s: State<Bomb>| s.read().phase == BombPhase::Racing;

    let app = app
        .handle(Stage::Pre, presence)
        .handle(Stage::Pre, game)
        .handle(Stage::Pre, clearer)
        .handle(Stage::Pre, ui)
        .handle(Stage::Update, on_connected)
        .handle(Stage::Update, on_disconnected)
        .handle(Stage::Update, on_setup_complete)
        .handle(Stage::Update, on_setup_aborted)
        .handle(Stage::Update, on_race_ended)
        .handle(Stage::Update, on_npl.run_if(while_racing))
        .handle(Stage::Update, on_player_left.run_if(while_racing))
        .handle(Stage::Update, on_toc.run_if(while_racing))
        .handle(Stage::Update, on_pit.run_if(while_racing))
        .handle(
            Stage::Update,
            on_player_teleported_to_pits.run_if(while_racing),
        )
        .handle(Stage::Update, on_crs.run_if(while_racing))
        .handle(Stage::Update, on_con.run_if(while_racing))
        .handle(Stage::Update, on_uco.run_if(while_racing))
        .handle(Stage::Update, on_tick.run_if(while_racing))
        .periodic(TICK_PERIOD, BombTick);

    let builder = insim::tcp(cfg.insim.addr)
        .isi_iname("bomb".to_string())
        .isi_prefix('!')
        .isi_admin_password(cfg.insim.admin_password);

    run(builder, app).await
}

/// Ad-hoc entry point: runs bomb without any DB backing.
pub async fn run_bomb(args: BombArgs) -> Result<(), AppError> {
    run_bomb_with(BombRunConfig {
        insim: args.insim,
        config: BombConfig {
            track: args.track,
            layout: args.layout,
            ..BombConfig::default()
        },
        db: None,
    })
    .await
}
