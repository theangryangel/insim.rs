//! Bomb mini-game subcommand. Ported from kitcar/examples/bomb.rs with
//! optional DB writes when `--db` and `--event-id` are provided.

mod chat;
mod config;
mod db;
mod events;
mod handlers;
mod state;
mod ui;

use std::time::Duration;

pub use config::{BombArgs, BombConfig, BombRunConfig};
use config::{MIN_PLAYERS, PENALTY_CLEAR_DELAY, TICK_PERIOD};
use events::BombTick;
use handlers::{
    on_axm, on_con, on_connected, on_crs, on_disconnected, on_pit, on_round_ended,
    on_round_started, on_run_ended, on_tick, on_uco,
};
use insim::insim::{IsiFlags, RaceLaps};
use kitcar::{
    App, AppError, ChatParser, HandlerExt, PenaltyClearer, RoundManager, RoundPolicy, RoundSpec,
    Stage, run,
};
use state::{ActiveRun, Bomb, BombGlobal};
use ui::{BombUi, BombView};

use crate::{
    components::{Dialog, Marquee},
    games::bomb::ui::BombMsg,
    run_registry::RunRegistry,
};

pub async fn run_bomb_with(cfg: BombRunConfig) -> Result<(), AppError> {
    // Capture rotation parameters before `cfg.config` is moved into the state.
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

    let app = App::<Bomb>::with_state(Bomb::new(cfg.config, cfg.db));
    let sender = app.sender().clone();

    let ui = BombUi::new(
        sender.clone(),
        BombGlobal::default(),
        |_ucid, invalidator| {
            let marquee = Marquee::new(invalidator.clone());
            let _tick_handle = tokio::spawn(async move {
                let mut interval = tokio::time::interval(Duration::from_millis(100));
                loop {
                    let _ = interval.tick().await;
                    invalidator.invalidate();
                }
            });
            BombView {
                _tick_handle,
                help: Dialog::default(),
                about: Dialog::default(),
                marquee,
            }
        },
    );

    let clearer = PenaltyClearer::new(PENALTY_CLEAR_DELAY);
    // Active runs, keyed by player and torn down by the world lifecycle: leaving
    // the track / disconnecting / tele-pitting all evict the run and emit
    // RunEnded (handled by `on_run_ended`).
    let runs = RunRegistry::<ActiveRun>::new().evict_on_pit();

    let while_racing = |r: RoundManager| r.is_racing();

    let app = app
        .handle(Stage::Pre, clearer)
        .with_ui(ui)
        .handle(Stage::Pre, rounds)
        .handle(Stage::Pre, runs)
        .handle(Stage::Update, ChatParser::<chat::Cmd>::new(&['!']))
        .handle(Stage::Update, on_connected)
        .handle(Stage::Update, on_disconnected)
        .handle(Stage::Update, on_axm)
        .handle(Stage::Update, on_round_started)
        .handle(Stage::Update, on_round_ended)
        // Ungated: a run evicted as the round ends must still be scored even
        // though the phase has already left Racing by the time this fires.
        .handle(Stage::Update, on_run_ended)
        .handle(Stage::Update, on_pit.run_if(while_racing))
        .handle(Stage::Update, on_crs.run_if(while_racing))
        .handle(Stage::Update, on_con.run_if(while_racing))
        .handle(Stage::Update, on_uco.run_if(while_racing))
        .handle(Stage::Update, on_tick.run_if(while_racing))
        .handle(Stage::Update, chat::handle_chat)
        .periodic(TICK_PERIOD, BombTick);

    let builder = insim::tcp(cfg.insim.addr)
        .isi_iname("bomb".to_string())
        .isi_prefix('!')
        .isi_admin_password(cfg.insim.admin_password)
        .isi_flags(IsiFlags::AXM_LOAD);

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
