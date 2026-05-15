//! Bomb mini-game implemented as a single long-lived async function.
//!
//! Demonstrates the "spawned async loop" pattern: no Scene trait, no
//! combinators. Just plain async-Rust over the `insim_app` primitives, with
//! the `Spawned<F>` handler doing all the wiring - it spawns the game task on
//! first dispatch and forwards every subsequent dispatch into the task's mpsc.
//!
//! The waterfall - wait → run → restart - is literally a `loop { ... }` in
//! `run_bomb`, with each phase as its own async helper. Cancellation, timeouts,
//! and round transitions fall out of standard `tokio::select!` patterns.
//!
//! ```text
//!     run_bomb               (outer loop)
//!       ├─ wait_for_players  (returns when >= 2 connections)
//!       └─ run_bomb_round    (returns when players drop or cancel fires)
//! ```
//!
//! Run with:
//!     cargo run -p insim_app --example bomb -- 127.0.0.1:29999
//!     cargo run -p insim_app --example bomb -- 127.0.0.1:29999 --admin-password hunter2

// Pulled in transitively by `insim_app`; silence unused-crate lint.
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use clap::Parser;
use fixedbitset as _;
use futures as _;
use insim::{
    Colour,
    core::object::{
        ObjectInfo,
        insim::{InsimCheckpoint, InsimCheckpointKind},
    },
    identifiers::{ConnectionId, PlayerId},
    insim::{BtnStyle, Con, Crs, Npl, Pit, PlayerType, Pll, Tiny, TinyType, Toc, Uco},
};
use insim_app::{
    App, AppError, Dispatch, Presence, Sender, serve, spawned,
    ui::{self, Component, InvalidateHandle, Ui},
    util::mtc,
};
use taffy as _;
use thiserror as _;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use tracing as _;

// ---------------------------------------------------------------------------
// Game tuning
// ---------------------------------------------------------------------------

const MIN_PLAYERS: usize = 2;
const TICK_PERIOD: Duration = Duration::from_millis(500);
const COLLISION_THRESHOLD_MPS: f32 = 30.0;

#[derive(Clone, Copy, Debug)]
struct BombConfig {
    checkpoint_timeout: Duration,
    checkpoint_penalty: Duration,
    collision_max_penalty: Duration,
}

impl Default for BombConfig {
    fn default() -> Self {
        Self {
            checkpoint_timeout: Duration::from_secs(30),
            checkpoint_penalty: Duration::from_millis(250),
            collision_max_penalty: Duration::from_millis(500),
        }
    }
}

// ---------------------------------------------------------------------------
// Game state (ported from examples/clockwork-carnage/.../bomb/state.rs,
// minus DB and per-player UI hooks)
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
struct ActiveRun {
    started_at: Instant,
    deadline: Instant,
    current_timeout: Duration,
    checkpoints: i64,
    uname: String,
    pname: String,
    ucid: ConnectionId,
}

impl ActiveRun {
    fn new(
        uname: String,
        pname: String,
        ucid: ConnectionId,
        config: &BombConfig,
        now: Instant,
    ) -> Self {
        Self {
            started_at: now,
            deadline: now + config.checkpoint_timeout,
            current_timeout: config.checkpoint_timeout,
            checkpoints: 0,
            uname,
            pname,
            ucid,
        }
    }

    fn survival_ms(&self, now: Instant) -> i64 {
        (self.deadline.min(now) - self.started_at).as_millis() as i64
    }

    fn time_left(&self, now: Instant) -> Duration {
        self.deadline.saturating_duration_since(now)
    }
}

#[derive(Clone, Debug)]
struct PlayerInfo {
    ucid: ConnectionId,
    pname: String,
    uname: String,
    ptype: PlayerType,
}

#[derive(Debug)]
enum CheckpointOutcome {
    Started {
        ucid: ConnectionId,
    },
    Refreshed {
        ucid: ConnectionId,
        checkpoints: i64,
        new_window: Duration,
    },
    Extended {
        ucid: ConnectionId,
        checkpoints: i64,
        time_left: Duration,
    },
}

struct BombState {
    config: BombConfig,
    active_runs: HashMap<PlayerId, ActiveRun>,
    /// In-memory leaderboard for the current round. Each entry is
    /// `(uname, checkpoints, survival_ms)`. Sorted by checkpoints desc,
    /// survival_ms desc.
    leaderboard: Vec<(String, i64, i64)>,
}

impl BombState {
    fn new(config: BombConfig) -> Self {
        Self {
            config,
            active_runs: HashMap::new(),
            leaderboard: Vec::new(),
        }
    }

    fn on_checkpoint(
        &mut self,
        p: &PlayerInfo,
        plid: PlayerId,
        is_finish: bool,
        now: Instant,
    ) -> Option<CheckpointOutcome> {
        if let Some(run) = self.active_runs.get_mut(&plid) {
            let outcome = if is_finish {
                run.current_timeout = self.config.checkpoint_timeout;
                run.deadline = now + self.config.checkpoint_timeout;
                run.checkpoints += 1;
                CheckpointOutcome::Refreshed {
                    ucid: run.ucid,
                    checkpoints: run.checkpoints,
                    new_window: self.config.checkpoint_timeout,
                }
            } else {
                run.deadline = now + run.current_timeout;
                run.current_timeout = run
                    .current_timeout
                    .saturating_sub(self.config.checkpoint_penalty);
                run.checkpoints += 1;
                CheckpointOutcome::Extended {
                    ucid: run.ucid,
                    checkpoints: run.checkpoints,
                    time_left: run.time_left(now),
                }
            };
            return Some(outcome);
        }
        if p.ptype.contains(PlayerType::AI) {
            return None;
        }
        let _ = self.active_runs.insert(
            plid,
            ActiveRun::new(p.uname.clone(), p.pname.clone(), p.ucid, &self.config, now),
        );
        Some(CheckpointOutcome::Started { ucid: p.ucid })
    }

    fn on_collision(
        &mut self,
        plid: PlayerId,
        speed_diff_mps: f32,
        now: Instant,
    ) -> Option<(ConnectionId, Duration, Duration)> {
        let fraction = (speed_diff_mps / COLLISION_THRESHOLD_MPS).clamp(0.0, 1.0);
        let penalty = Duration::from_millis(
            (fraction * self.config.collision_max_penalty.as_millis() as f32) as u64,
        );
        if penalty.is_zero() {
            return None;
        }
        let run = self.active_runs.get_mut(&plid)?;
        run.deadline = run.deadline.checked_sub(penalty).unwrap_or(now);
        Some((run.ucid, penalty, run.time_left(now)))
    }

    fn on_reset(
        &mut self,
        plid: PlayerId,
        now: Instant,
    ) -> Option<(ConnectionId, Duration, Duration)> {
        let penalty = self.config.checkpoint_penalty;
        let run = self.active_runs.get_mut(&plid)?;
        run.deadline = run.deadline.checked_sub(penalty).unwrap_or(now);
        Some((run.ucid, penalty, run.time_left(now)))
    }

    fn finalize(&mut self, run: &ActiveRun, survival_ms: i64) {
        self.leaderboard
            .push((run.uname.clone(), run.checkpoints, survival_ms));
        self.leaderboard
            .sort_by(|a, b| b.1.cmp(&a.1).then(b.2.cmp(&a.2)));
        self.leaderboard.truncate(10);
    }

    fn tick_expired(&mut self, now: Instant) -> Vec<ActiveRun> {
        let expired: Vec<PlayerId> = self
            .active_runs
            .iter()
            .filter(|(_, r)| r.deadline < now)
            .map(|(k, _)| *k)
            .collect();
        expired
            .into_iter()
            .filter_map(|k| self.active_runs.remove(&k))
            .collect()
    }

    fn snapshot(&self, phase: &str) -> BombGlobal {
        let now = Instant::now();
        let mut active: Vec<_> = self
            .active_runs
            .values()
            .map(|r| (r.pname.clone(), r.checkpoints, r.time_left(now)))
            .collect();
        active.sort_by_key(|r| std::cmp::Reverse(r.1));
        BombGlobal {
            phase: phase.to_string(),
            leaderboard: self.leaderboard.clone(),
            active_runs: active,
        }
    }
}

// ---------------------------------------------------------------------------
// UI - single fixed view defined at the top level, wired exactly like smoke.rs
// ---------------------------------------------------------------------------

#[derive(Clone, Default, Debug)]
struct BombGlobal {
    phase: String,
    leaderboard: Vec<(String, i64, i64)>,
    /// (pname, checkpoints, time_left)
    active_runs: Vec<(String, i64, Duration)>,
}

#[derive(Clone, Debug)]
enum BombMsg {}

struct BombView {
    _invalidator: InvalidateHandle,
}

impl Component for BombView {
    type Message = BombMsg;
    type Props<'a> = (&'a BombGlobal, &'a ());

    fn render(&self, (global, _): Self::Props<'_>) -> ui::Node<Self::Message> {
        let active_rows: Vec<_> = global
            .active_runs
            .iter()
            .map(|(p, cps, time_left)| {
                ui::text(
                    format!("{p}: {cps} cps  {:.1}s", time_left.as_secs_f64()),
                    BtnStyle::default(),
                )
                .w(50.0)
                .h(5.0)
            })
            .collect();

        let leaderboard_rows: Vec<_> = global
            .leaderboard
            .iter()
            .take(5)
            .enumerate()
            .map(|(i, (u, cps, ms))| {
                ui::text(
                    format!("{}. {u}: {cps} cps ({:.1}s)", i + 1, *ms as f64 / 1000.0),
                    BtnStyle::default(),
                )
                .w(50.0)
                .h(5.0)
            })
            .collect();

        ui::container()
            .flex()
            .flex_col()
            .with_child(
                ui::text(format!("Bomb - {}", global.phase), BtnStyle::default())
                    .w(50.0)
                    .h(5.0),
            )
            .with_child(ui::text("Active runs", BtnStyle::default()).w(50.0).h(5.0))
            .with_children(active_rows)
            .with_child(ui::text("Top runs", BtnStyle::default()).w(50.0).h(5.0))
            .with_children(leaderboard_rows)
    }
}

// ---------------------------------------------------------------------------
// Game loop
// ---------------------------------------------------------------------------

struct BombCtx {
    rx: mpsc::UnboundedReceiver<Dispatch>,
    sender: Sender,
    presence: Presence,
    ui: Ui<BombView, BombGlobal, ()>,
    cancel: CancellationToken,
    config: BombConfig,
}

/// Phase 1: wait until at least `min` connections are present.
/// Returns early on cancellation.
async fn wait_for_players(ctx: &mut BombCtx, min: usize) {
    ctx.ui.assign(BombGlobal {
        phase: format!("waiting for {min} players"),
        ..Default::default()
    });
    let _ = ctx.sender.packet(mtc(
        format!("Bomb - waiting for {min} players..."),
        Some(ConnectionId::ALL),
    ));
    loop {
        if ctx.presence.count() >= min {
            return;
        }
        tokio::select! {
            _ = ctx.cancel.cancelled() => return,
            // Any forwarded packet wakes us up. Presence's `on_event` runs
            // before the forwarder, so by the time we re-check count here,
            // an Ncn/Cnl has already updated the connection map.
            res = ctx.rx.recv() => {
                if res.is_none() { return; }
            }
        }
    }
}

/// Phase 2: run rounds until players drop or cancel fires.
async fn run_bomb_round(ctx: &mut BombCtx) {
    let mut state = BombState::new(ctx.config);
    let mut players: HashMap<PlayerId, PlayerInfo> = HashMap::new();
    let mut tick = tokio::time::interval(TICK_PERIOD);
    tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    let _ = ctx.sender.packet(mtc(
        format!(
            "Bomb - hit checkpoints before the {:.0}s timer expires!",
            ctx.config.checkpoint_timeout.as_secs_f64()
        ),
        Some(ConnectionId::ALL),
    ));

    // Ask LFS to send us a fresh `Npl` for every player currently in the
    // race; they'll arrive as Packet::Npl on the forwarder channel.
    let _ = ctx.sender.packet(insim::Packet::Tiny(Tiny {
        subt: TinyType::Npl,
        ..Default::default()
    }));

    ctx.ui.assign(state.snapshot("running"));

    loop {
        if ctx.presence.count() < MIN_PLAYERS {
            let _ = ctx.sender.packet(mtc(
                "Bomb - not enough players, restarting.",
                Some(ConnectionId::ALL),
            ));
            return;
        }

        tokio::select! {
            _ = ctx.cancel.cancelled() => return,

            // Periodic deadline check - runs that overshot their bomb timer get popped here.
            _ = tick.tick() => {
                let now = Instant::now();
                let expired = state.tick_expired(now);
                let had_expired = !expired.is_empty();
                for run in expired {
                    let survival_ms = run.survival_ms(now);
                    let msg = format!(
                        "BOOM - {} cps, {:.1}s",
                        run.checkpoints,
                        survival_ms as f64 / 1000.0
                    )
                    .red();
                    let _ = ctx.sender.packet(mtc(msg, Some(run.ucid)));
                    state.finalize(&run, survival_ms);
                }
                // Refresh UI either when a run expired (state changed) or
                // periodically so the time-left numbers tick down visibly.
                if had_expired || !state.active_runs.is_empty() {
                    ctx.ui.assign(state.snapshot("running"));
                }
            }

            res = ctx.rx.recv() => {
                let Some(d) = res else { return; };
                handle_packet(d, &mut state, &mut players, ctx);
            }
        }
    }
}

fn handle_packet(
    d: Dispatch,
    state: &mut BombState,
    players: &mut HashMap<PlayerId, PlayerInfo>,
    ctx: &BombCtx,
) {
    let now = Instant::now();
    let Dispatch::Packet(p) = d else {
        return;
    };
    match p {
        insim::Packet::Npl(Npl {
            plid,
            ucid,
            ptype,
            pname,
            ..
        }) => {
            let uname = ctx.presence.get(ucid).map(|c| c.uname).unwrap_or_default();
            let _ = players.insert(
                plid,
                PlayerInfo {
                    ucid,
                    pname,
                    uname,
                    ptype,
                },
            );
        },
        insim::Packet::Pll(Pll { plid, .. }) => {
            let _ = players.remove(&plid);
            if let Some(run) = state.active_runs.remove(&plid) {
                let survival_ms = run.survival_ms(now);
                let _ = ctx.sender.packet(mtc(
                    format!("Run ended - left race after {} cps", run.checkpoints).red(),
                    Some(run.ucid),
                ));
                state.finalize(&run, survival_ms);
                ctx.ui.assign(state.snapshot("running"));
            }
        },
        insim::Packet::Toc(Toc { plid, newucid, .. }) => {
            if let Some(p) = players.get_mut(&plid) {
                p.ucid = newucid;
            }
            if let Some(r) = state.active_runs.get_mut(&plid) {
                r.ucid = newucid;
            }
        },
        insim::Packet::Pit(Pit { plid, .. }) => {
            if let Some(run) = state.active_runs.remove(&plid) {
                let survival_ms = run.survival_ms(now);
                let _ = ctx.sender.packet(mtc(
                    format!(
                        "PITTED - run ended after {} cps. Commit to your fuel.",
                        run.checkpoints
                    )
                    .red(),
                    Some(run.ucid),
                ));
                state.finalize(&run, survival_ms);
                ctx.ui.assign(state.snapshot("running"));
            }
        },
        insim::Packet::Crs(Crs { plid, .. }) => {
            if let Some((ucid, penalty, time_left)) = state.on_reset(plid, now) {
                let _ = ctx.sender.packet(mtc(
                    format!(
                        "PENALTY -{:.2}s - {:.1}s left",
                        penalty.as_secs_f64(),
                        time_left.as_secs_f64()
                    )
                    .red(),
                    Some(ucid),
                ));
                ctx.ui.assign(state.snapshot("running"));
            }
        },
        insim::Packet::Con(Con { spclose, a, b, .. }) => {
            let mps = spclose.to_meters_per_sec();
            for plid in [a.plid, b.plid] {
                if let Some((ucid, penalty, time_left)) = state.on_collision(plid, mps, now) {
                    let _ = ctx.sender.packet(mtc(
                        format!(
                            "PENALTY -{:.2}s - {:.1}s left",
                            penalty.as_secs_f64(),
                            time_left.as_secs_f64()
                        )
                        .red(),
                        Some(ucid),
                    ));
                    ctx.ui.assign(state.snapshot("running"));
                }
            }
        },
        insim::Packet::Uco(Uco {
            info: ObjectInfo::InsimCheckpoint(InsimCheckpoint { kind, .. }),
            plid,
            ..
        }) => {
            let Some(player) = players.get(&plid).cloned() else {
                return;
            };
            let is_finish = matches!(kind, InsimCheckpointKind::Finish);
            if let Some(outcome) = state.on_checkpoint(&player, plid, is_finish, now) {
                match outcome {
                    CheckpointOutcome::Started { ucid } => {
                        let _ = ctx.sender.packet(mtc(
                            "Run started - hit every checkpoint!".light_green(),
                            Some(ucid),
                        ));
                    },
                    CheckpointOutcome::Refreshed {
                        ucid,
                        checkpoints,
                        new_window,
                    } => {
                        let _ = ctx.sender.packet(mtc(
                            format!(
                                "FINISH - cp {checkpoints} - REFRESHED {:.1}s",
                                new_window.as_secs_f64()
                            )
                            .yellow(),
                            Some(ucid),
                        ));
                    },
                    CheckpointOutcome::Extended {
                        ucid,
                        checkpoints,
                        time_left,
                    } => {
                        let _ = ctx.sender.packet(mtc(
                            format!("cp {checkpoints} - {:.1}s left", time_left.as_secs_f64())
                                .light_green(),
                            Some(ucid),
                        ));
                    },
                }
                ctx.ui.assign(state.snapshot("running"));
            }
        },
        _ => {},
    }
}

/// The whole game.
///
/// Notice what isn't here: no scene combinators, no state-machine enum, no
/// modal-takeover semantics. The flow reads top-to-bottom and uses ordinary
/// async-Rust idioms for cancellation (`tokio::select!` on `cancel.cancelled()`)
/// and round restart (`continue` in the outer loop).
async fn run_bomb(mut ctx: BombCtx) {
    loop {
        if ctx.cancel.is_cancelled() {
            return;
        }
        wait_for_players(&mut ctx, MIN_PLAYERS).await;
        if ctx.cancel.is_cancelled() {
            return;
        }
        run_bomb_round(&mut ctx).await;
    }
}

// ---------------------------------------------------------------------------
// Wiring
// ---------------------------------------------------------------------------

#[derive(Parser, Debug)]
#[command(about = "insim_app - bomb game example (no scenes)")]
struct Args {
    /// LFS InSim address (host:port).
    #[arg(long, default_value = "127.0.0.1:29999")]
    addr: String,

    /// InSim admin password, if the host requires one.
    #[arg(long)]
    admin_password: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), AppError> {
    tracing_subscriber::fmt::init();
    let args = Args::parse();

    let app = App::<()>::new();

    // UI lives at the top level, exactly like smoke.rs. Types are known at
    // compile time because the game type is known at compile time.
    let ui = Ui::<BombView, BombGlobal, ()>::new(
        app.sender().clone(),
        BombGlobal::default(),
        |_ucid, invalidator| BombView {
            _invalidator: invalidator,
        },
    );

    let presence = Presence::new(app.sender().clone());
    let cancel = app.cancel_token().clone();
    let config = BombConfig::default();

    // `spawned(...)` handles both halves of the pumped-task pattern: it
    // spawns `run_bomb` on its first dispatch and forwards every dispatch
    // into the mpsc the game task drains. Extensions are wired in via
    // ordinary closure capture - they're all `Clone`.
    let app = app
        .with_state(())
        .extension(presence.clone())
        .extension(ui.clone())
        .handler(spawned({
            let presence = presence.clone();
            let ui = ui.clone();
            let cancel = cancel.clone();
            move |rx, sender| {
                run_bomb(BombCtx {
                    rx,
                    sender,
                    presence,
                    ui,
                    cancel,
                    config,
                })
            }
        }));

    let builder = insim::tcp(args.addr)
        .isi_iname("bomb".to_string())
        .isi_prefix('!')
        .isi_admin_password(args.admin_password);

    serve(builder, app).await
}
